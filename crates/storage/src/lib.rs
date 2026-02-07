mod backends;
mod builders;
mod error;

pub use backends::DelayStorage;
pub use backends::DiskStorage;
pub use backends::EncryptedStorage;
pub use backends::MemStorage;
pub use backends::Object;
pub use backends::Options;

use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

pub use async_tempfile::TempFile;

pub use builders::{CoverFileBuilder, FileBuilder, MangaPageFileBuilder, UserCoverFileBuilder};
pub use error::StorageError;
use futures_util::{StreamExt as _, TryStreamExt as _};
use rand::prelude::IndexedRandom;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt as _, AsyncSeekExt as _, AsyncWriteExt},
    sync::{Mutex, Notify, Semaphore},
};
use tokio_util::io::ReaderStream;

use crate::{
    backends::{ByteStream, StorageReader, StorageWriter},
    error::{ProcessingError, StorageResult},
};

pub struct StorageSystem {
    files: Arc<Mutex<HashMap<String, StoredFile>>>,
    transcode_sem: Arc<Semaphore>,
    writer: Arc<dyn StorageWriter + Send + Sync>,
    pub reader: Arc<dyn StorageReader + Send + Sync>,
    path: PathBuf,
}

struct StoredFile {
    ext: Option<(&'static str, &'static str)>,
    dims: Option<(u32, u32)>,
    state: EntryState,
}

enum EntryState {
    Processing { notify: Arc<Notify> },
    Uploaded { handle: String },
    Failed { error: ProcessingError },
}

impl StoredFile {
    fn new_processing(ext: Option<(&'static str, &'static str)>, notify: Arc<Notify>) -> Self {
        Self {
            ext,
            dims: None,
            state: EntryState::Processing { notify },
        }
    }

    fn mark_uploaded(
        &mut self,
        handle: String,
        dims: Option<(u32, u32)>,
        ext: Option<(&'static str, &'static str)>,
    ) {
        self.dims = dims;
        self.ext = ext;
        self.state = EntryState::Uploaded { handle };
    }

    fn mark_failed(&mut self, error: ProcessingError) {
        self.state = EntryState::Failed { error };
    }
}

pub trait FileBuilderExt {
    fn width(&self) -> Option<u32>;
    fn height(&self) -> Option<u32>;
    fn ext(&self) -> StorageResult<&str>;
}

pub fn file_to_bytestream(file: File) -> ByteStream {
    ReaderStream::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed()
}

async fn get_random_image(folder: &Path) -> Option<PathBuf> {
    let extensions = ["png", "gif", "jpeg", "jpg", "qoi", "avif"];

    let mut files: Vec<PathBuf> = Vec::new();
    let mut dir = tokio::fs::read_dir(folder).await.ok()?;

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if file_name.starts_with('.') {
            continue;
        }

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_ascii_lowercase(),
            None => continue,
        };

        if extensions.contains(&ext.as_str()) {
            files.push(path);
        }
    }
    files.choose(&mut rand::rng()).cloned()
}

#[derive(Debug, Clone)]
pub struct FileId(String);

impl FileId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    pub fn inner(self) -> String {
        self.0
    }
    pub fn inner_ref(&self) -> &str {
        &self.0
    }
}

const CHAPTER_MAGIC: &[u8; 8] = b"MRCHAP01";
const MANGA_MAGIC: &[u8; 8] = b"MRMANG01";

#[derive(Debug, Clone)]
pub enum RegisterTempResult {
    File(FileId),
    Chapter(Vec<FileId>),
    Manga(RegisteredMangaTemp),
}

#[derive(Debug, Clone)]
pub struct RegisteredMangaTemp {
    pub metadata: FileId,
    pub images: Vec<FileId>,
    pub chapter_image_indexes: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MangaBundleMetadata {
    pub chapter_image_indexes: Vec<Vec<u32>>,
}

enum TempContainerPayload {
    SingleFile(TempFile),
    Chapter(Vec<Vec<u8>>),
    Manga {
        metadata_bytes: Vec<u8>,
        metadata: MangaBundleMetadata,
        images: Vec<Vec<u8>>,
    },
}

fn read_u32_le(cur: &mut Cursor<&[u8]>) -> Result<u32, std::io::Error> {
    let mut buf = [0u8; 4];
    std::io::Read::read_exact(cur, &mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_blob(cur: &mut Cursor<&[u8]>) -> Result<Vec<u8>, std::io::Error> {
    let len = read_u32_le(cur)? as usize;
    let pos = cur.position() as usize;
    let all = cur.get_ref();
    if pos + len > all.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "invalid container payload length",
        ));
    }
    let out = all[pos..pos + len].to_vec();
    cur.set_position((pos + len) as u64);
    Ok(out)
}

fn parse_chapter_container(data: &[u8]) -> Result<Vec<Vec<u8>>, std::io::Error> {
    let mut cur = Cursor::new(data);
    let mut magic = [0u8; 8];
    std::io::Read::read_exact(&mut cur, &mut magic)?;
    if &magic != CHAPTER_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid chapter magic",
        ));
    }

    let count = read_u32_le(&mut cur)? as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(read_blob(&mut cur)?);
    }
    Ok(out)
}

fn parse_manga_container(
    data: &[u8],
) -> Result<(Vec<u8>, MangaBundleMetadata, Vec<Vec<u8>>), std::io::Error> {
    let mut cur = Cursor::new(data);
    let mut magic = [0u8; 8];
    std::io::Read::read_exact(&mut cur, &mut magic)?;
    if &magic != MANGA_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid manga magic",
        ));
    }

    let metadata_bytes = read_blob(&mut cur)?;
    let metadata: MangaBundleMetadata =
        bincode::deserialize(&metadata_bytes).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid metadata: {e}"))
        })?;

    let count = read_u32_le(&mut cur)? as usize;
    let mut images = Vec::with_capacity(count);
    for _ in 0..count {
        images.push(read_blob(&mut cur)?);
    }

    for chapter in &metadata.chapter_image_indexes {
        for idx in chapter {
            if (*idx as usize) >= images.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "metadata references out-of-range image index",
                ));
            }
        }
    }

    Ok((metadata_bytes, metadata, images))
}

async fn split_pdf_to_png_pages(path: &Path) -> Result<Vec<Vec<u8>>, ProcessingError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<Vec<Vec<u8>>, std::io::Error> {
        let workdir = std::env::temp_dir().join(format!("storage_pdf_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&workdir)?;
        let prefix = workdir.join("page");

        let output = std::process::Command::new("pdftoppm")
            .arg("-png")
            .arg(&path)
            .arg(&prefix)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = std::fs::remove_dir_all(&workdir);
            return Err(std::io::Error::other(format!(
                "pdftoppm failed: {}",
                stderr.trim()
            )));
        }

        let mut pages: Vec<_> = std::fs::read_dir(&workdir)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
            .collect();
        pages.sort();

        if pages.is_empty() {
            let _ = std::fs::remove_dir_all(&workdir);
            return Err(std::io::Error::other(
                "pdftoppm produced no pages for pdf input",
            ));
        }

        let mut out = Vec::with_capacity(pages.len());
        for page in pages {
            out.push(std::fs::read(page)?);
        }
        let _ = std::fs::remove_dir_all(&workdir);
        Ok(out)
    })
    .await
    .map_err(ProcessingError::PdfWorkerJoin)?
    .map_err(ProcessingError::SplitPdf)
}

async fn detect_ext(path: &Path) -> Option<(&'static str, &'static str)> {
    let file = File::open(path).await.ok()?;

    let limit = file
        .metadata()
        .await
        .map(|m| std::cmp::min(m.len(), 8192) as usize + 1)
        .unwrap_or(0);
    let mut bytes = Vec::with_capacity(limit);
    file.take(8192).read_to_end(&mut bytes).await.ok()?;
    let kind = infer::get(&bytes)?;
    let ext = kind.extension();
    if ext == "jpg" {
        Some((kind.mime_type(), "jpeg"))
    } else {
        Some((kind.mime_type(), kind.extension()))
    }
}

impl StorageSystem {
    pub async fn new_temp_file(&self) -> StorageResult<TempFile> {
        Ok(TempFile::new().await?)
    }

    async fn temp_file_from_bytes(&self, bytes: &[u8]) -> StorageResult<TempFile> {
        let mut tf = TempFile::new().await?;
        tf.write_all(bytes).await?;
        tf.flush().await?;
        tf.sync_all().await?;
        Ok(tf)
    }

    async fn detect_container(&self, mut tf: TempFile) -> StorageResult<TempContainerPayload> {
        tf.flush().await?;
        tf.sync_all().await?;
        tf.seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(StorageError::Io)?;

        let mut magic = [0u8; 8];
        let mut read = 0usize;
        while read < magic.len() {
            let n = tf
                .read(&mut magic[read..])
                .await
                .map_err(StorageError::Io)?;
            if n == 0 {
                break;
            }
            read += n;
        }

        tf.seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(StorageError::Io)?;

        if read < magic.len() {
            return Ok(TempContainerPayload::SingleFile(tf));
        }

        if &magic == CHAPTER_MAGIC {
            let mut data = vec![];
            tf.read_to_end(&mut data).await.map_err(StorageError::Io)?;
            let images = parse_chapter_container(&data).map_err(StorageError::Io)?;
            return Ok(TempContainerPayload::Chapter(images));
        }

        if &magic == MANGA_MAGIC {
            let mut data = vec![];
            tf.read_to_end(&mut data).await.map_err(StorageError::Io)?;
            let (metadata_bytes, metadata, images) =
                parse_manga_container(&data).map_err(StorageError::Io)?;
            return Ok(TempContainerPayload::Manga {
                metadata_bytes,
                metadata,
                images,
            });
        }

        if matches!(detect_ext(tf.file_path()).await, Some((mime, _)) if mime == "application/pdf") {
            let pages = split_pdf_to_png_pages(tf.file_path())
                .await
                .map_err(StorageError::Processing)?;
            return Ok(TempContainerPayload::Chapter(pages));
        }

        Ok(TempContainerPayload::SingleFile(tf))
    }

    async fn register_single_temp_file(&self, mut tf: TempFile) -> StorageResult<FileId> {
        tf.flush().await?;
        tf.sync_all().await?;

        let id = uuid::Uuid::new_v4().to_string();

        let ext = detect_ext(tf.file_path()).await;
        let is_image = ext.map(|(m, _)| m.starts_with("image/")).unwrap_or(false);
        let allowed = matches!(
            ext.map(|(_, e)| e),
            Some("avif" | "webp" | "png" | "jpeg" | "jpg" | "gif")
        );

        let notify = Arc::new(Notify::new());

        {
            let mut map = self.files.lock().await;
            map.insert(id.clone(), StoredFile::new_processing(ext, notify.clone()));
        }

        let files = self.files.clone();
        let sem = self.transcode_sem.clone();
        let writer = self.writer.clone();
        let id2 = id.clone();

        tokio::spawn(async move {
            let notify_to_wake = notify.clone();
            let result: Result<
                (
                    String,
                    Option<(u32, u32)>,
                    Option<(&'static str, &'static str)>,
                ),
                ProcessingError,
            > = async {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|_| ProcessingError::SemaphoreClosed)?;

                let mut final_ext = ext;
                let mut dims = None;
                let upload_handle = format!("temp/{}", uuid::Uuid::new_v4());
                if is_image && !allowed {
                    tf.seek(std::io::SeekFrom::Start(0))
                        .await
                        .map_err(ProcessingError::SeekTempFile)?;
                    let mut buffer = vec![];
                    tf.read_to_end(&mut buffer)
                        .await
                        .map_err(ProcessingError::ReadTempFile)?;

                    let (jpeg_bytes, size) = tokio::task::spawn_blocking(move || {
                        let img = image::load_from_memory(&buffer)?;
                        let mut out = Vec::new();
                        let size = (img.width(), img.height());
                        let rgb = img.to_rgb8();
                        let mut enc = image::codecs::jpeg::JpegEncoder::new(&mut out);
                        enc.encode(
                            &rgb,
                            rgb.width(),
                            rgb.height(),
                            image::ColorType::Rgb8.into(),
                        )?;
                        Ok::<_, image::ImageError>((out, size))
                    })
                    .await
                    .map_err(ProcessingError::ImageWorkerJoin)?
                    .map_err(ProcessingError::ImageConversion)?;

                    dims = Some(size);
                    final_ext = Some(("image/jpeg", "jpeg"));

                    let stream = futures_util::stream::once(async move {
                        Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from(jpeg_bytes))
                    })
                    .boxed();
                    writer
                        .write(&upload_handle, None, stream)
                        .await
                        .map_err(ProcessingError::UploadConverted)?;
                } else {
                    if is_image {
                        let f = tf.file_path().to_owned();
                        dims = Some(
                            tokio::task::spawn_blocking(move || {
                                match image::image_dimensions(&f) {
                                    Ok(v) => Ok(v),
                                    Err(_) => {
                                        let file = std::fs::File::open(&f)?;
                                        let reader = std::io::BufReader::new(file);
                                        image::ImageReader::new(reader)
                                            .with_guessed_format()?
                                            .into_dimensions()
                                    }
                                }
                            })
                            .await
                            .map_err(ProcessingError::DimensionWorkerJoin)?
                            .map_err(ProcessingError::ReadImageDimensions)?,
                        );
                    }

                    let f = File::open(tf.file_path())
                        .await
                        .map_err(ProcessingError::OpenTempFileForUpload)?;
                    writer
                        .write(&upload_handle, None, file_to_bytestream(f))
                        .await
                        .map_err(ProcessingError::UploadTemp)?;
                }

                Ok((upload_handle, dims, final_ext))
            }
            .await;

            let mut map = files.lock().await;
            let Some(entry) = map.get_mut(&id2) else {
                notify_to_wake.notify_waiters();
                return;
            };

            match result {
                Ok((handle, dims, ext)) => {
                    entry.mark_uploaded(handle, dims, ext);
                }
                Err(error) => {
                    entry.mark_failed(error);
                }
            }

            notify_to_wake.notify_waiters();
        });

        Ok(FileId::new(id))
    }

    pub async fn register_temp_file(&self, tf: TempFile) -> StorageResult<RegisterTempResult> {
        match self.detect_container(tf).await? {
            TempContainerPayload::SingleFile(tf) => self
                .register_single_temp_file(tf)
                .await
                .map(RegisterTempResult::File),
            TempContainerPayload::Chapter(images) => {
                let mut out = Vec::with_capacity(images.len());
                for image in images {
                    let tf = self.temp_file_from_bytes(&image).await?;
                    out.push(self.register_single_temp_file(tf).await?);
                }
                Ok(RegisterTempResult::Chapter(out))
            }
            TempContainerPayload::Manga {
                metadata_bytes,
                metadata,
                images,
            } => {
                let metadata_tf = self.temp_file_from_bytes(&metadata_bytes).await?;
                let metadata_id = self.register_single_temp_file(metadata_tf).await?;

                let mut image_ids = Vec::with_capacity(images.len());
                for image in images {
                    let tf = self.temp_file_from_bytes(&image).await?;
                    image_ids.push(self.register_single_temp_file(tf).await?);
                }

                Ok(RegisterTempResult::Manga(RegisteredMangaTemp {
                    metadata: metadata_id,
                    images: image_ids,
                    chapter_image_indexes: metadata.chapter_image_indexes,
                }))
            }
        }
    }

    pub async fn take(&self, id: FileId) -> StorageResult<FileBuilder> {
        loop {
            let wait_on = {
                let mut map = self.files.lock().await;
                let Some(entry) = map.get(id.inner_ref()) else {
                    return Err(StorageError::HandleNotFound);
                };

                match &entry.state {
                    EntryState::Uploaded { handle } => {
                        let handle = handle.clone();
                        let entry = map.remove(id.inner_ref()).unwrap();
                        return Ok(FileBuilder {
                            dims: entry.dims,
                            ext: entry.ext,
                            temp_id: handle,
                            target_id: PathBuf::new(),
                            allowed_drop: false,
                            writer: self.writer.clone(),
                        });
                    }
                    EntryState::Processing { notify } => Some(notify.clone()),
                    EntryState::Failed { .. } => {
                        let error = match map.remove(id.inner_ref()) {
                            Some(stored) => match stored.state {
                                EntryState::Failed { error } => error,
                                _ => {
                                    return Err(StorageError::Io(std::io::Error::other(
                                        "invalid state transition",
                                    )));
                                }
                            },
                            None => return Err(StorageError::HandleNotFound),
                        };
                        return Err(StorageError::Processing(error));
                    }
                }
            };

            if let Some(n) = wait_on {
                n.notified().await;
            }
        }
    }

    pub async fn get_user_cover(
        &self,
        id: Option<FileId>,
        options: Option<Options>,
    ) -> StorageResult<UserCoverFileBuilder> {
        let item = match id {
            Some(id) => self.take(id).await.map(UserCoverFileBuilder::from),
            None => {
                let p = self.path.join("cover_templates");
                let ri = get_random_image(&p)
                    .await
                    .ok_or(StorageError::NoDefaultImageAvailable)?;
                let f = File::open(&p).await?;
                let id = PathBuf::from(format!("temp/{}", uuid::Uuid::new_v4()));
                let id = id.to_string_lossy();
                self.writer
                    .write(&id, options, file_to_bytestream(f))
                    .await?;
                Ok(UserCoverFileBuilder::from(FileBuilder {
                    dims: None,
                    allowed_drop: false,
                    temp_id: id.to_string(),
                    ext: detect_ext(&ri).await,
                    target_id: PathBuf::new(),
                    writer: self.writer.clone(),
                }))
            }
        }?;
        Ok(item)
    }

    pub async fn new<T: StorageReader + StorageWriter + Send + Sync>(
        path: &Path,
        rw: Arc<T>,
    ) -> std::io::Result<Self> {
        Ok(StorageSystem {
            reader: rw.clone(),
            writer: rw,
            path: path.to_path_buf(),
            files: Arc::default(),
            transcode_sem: Arc::new(Semaphore::new(5)),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, sync::Arc};

    use image::{DynamicImage, ImageFormat};
    use tokio::io::AsyncWriteExt as _;

    use crate::{
        CoverFileBuilder, FileId, MangaBundleMetadata, MemStorage, RegisterTempResult,
        StorageError, StorageSystem, CHAPTER_MAGIC, MANGA_MAGIC,
    };

    async fn read_all_bytes(
        reader: &Arc<dyn crate::backends::StorageReader + Send + Sync>,
        key: &str,
    ) -> Vec<u8> {
        use futures_util::StreamExt as _;

        let mut out = Vec::new();
        let mut stream = reader
            .get(key, None)
            .await
            .expect("expected object to exist")
            .stream;

        while let Some(chunk) = stream.next().await {
            out.extend_from_slice(&chunk.expect("stream should not fail"));
        }

        out
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut cursor = cursor;
        DynamicImage::new_rgba8(width, height)
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("png encode should succeed");
        let out = cursor.into_inner();
        assert_eq!(
            infer::get(&out).map(|k| k.mime_type()),
            Some("image/png"),
            "generated png bytes should be valid"
        );
        out
    }

    fn bmp_bytes(width: u32, height: u32) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut cursor = cursor;
        DynamicImage::new_rgba8(width, height)
            .write_to(&mut cursor, ImageFormat::Bmp)
            .expect("bmp encode should succeed");
        let out = cursor.into_inner();
        assert_eq!(
            infer::get(&out).map(|k| k.mime_type()),
            Some("image/bmp"),
            "generated bmp bytes should be valid"
        );
        out
    }

    fn make_chapter_container(images: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(CHAPTER_MAGIC);
        out.extend_from_slice(&(images.len() as u32).to_le_bytes());
        for image in images {
            out.extend_from_slice(&(image.len() as u32).to_le_bytes());
            out.extend_from_slice(image);
        }
        out
    }

    fn make_manga_container(metadata: &MangaBundleMetadata, images: &[Vec<u8>]) -> Vec<u8> {
        let metadata_bytes = bincode::serialize(metadata).expect("metadata serialize");
        let mut out = Vec::new();
        out.extend_from_slice(MANGA_MAGIC);
        out.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&metadata_bytes);
        out.extend_from_slice(&(images.len() as u32).to_le_bytes());
        for image in images {
            out.extend_from_slice(&(image.len() as u32).to_le_bytes());
            out.extend_from_slice(image);
        }
        out
    }

    fn unwrap_storage<T>(res: Result<T, StorageError>, ctx: &str) -> T {
        match res {
            Ok(v) => v,
            Err(StorageError::HandleNotFound) => panic!("{ctx}: handle not found"),
            Err(StorageError::NoDefaultImageAvailable) => panic!("{ctx}: no default image"),
            Err(StorageError::MissingExtension) => panic!("{ctx}: missing extension"),
            Err(StorageError::Processing(e)) => panic!("{ctx}: processing error: {e}"),
            Err(StorageError::Io(e)) => panic!("{ctx}: io error: {e}"),
            Err(StorageError::TempFile(e)) => panic!("{ctx}: tempfile error: {e}"),
        }
    }

    fn unwrap_single_register(res: Result<RegisterTempResult, StorageError>, ctx: &str) -> FileId {
        match unwrap_storage(res, ctx) {
            RegisterTempResult::File(id) => id,
            RegisterTempResult::Chapter(_) => panic!("{ctx}: expected single file, got chapter"),
            RegisterTempResult::Manga(_) => panic!("{ctx}: expected single file, got manga"),
        }
    }

    #[tokio::test]
    async fn register_take_plain_file_roundtrip() {
        let backend = Arc::new(MemStorage::new());
        let storage = match StorageSystem::new(std::env::temp_dir().as_path(), backend).await {
            Ok(v) => v,
            Err(e) => panic!("storage init failed: {e}"),
        };

        let payload = b"plain-register-take-roundtrip".to_vec();
        let mut tf = storage
            .new_temp_file()
            .await
            .unwrap_or_else(|_| panic!("tempfile create failed"));
        tf.write_all(&payload)
            .await
            .expect("write to tempfile should succeed");
        tf.sync_all().await.expect("sync should succeed");

        let id = unwrap_single_register(storage.register_temp_file(tf).await, "register");
        let fb = unwrap_storage(storage.take(id).await, "take");
        CoverFileBuilder::from(fb)
            .build("plain-roundtrip")
            .await
            .unwrap_or_else(|_| panic!("build failed"));

        let got = read_all_bytes(&storage.reader, "covers/plain-roundtrip").await;
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn register_take_supported_image_keeps_ext_and_dims() {
        let backend = Arc::new(MemStorage::new());
        let storage = match StorageSystem::new(std::env::temp_dir().as_path(), backend).await {
            Ok(v) => v,
            Err(e) => panic!("storage init failed: {e}"),
        };

        let payload = png_bytes(3, 2);
        let mut tf = storage
            .new_temp_file()
            .await
            .unwrap_or_else(|_| panic!("tempfile create failed"));
        tf.write_all(&payload)
            .await
            .expect("write to tempfile should succeed");
        tf.sync_all().await.expect("sync should succeed");

        let id = unwrap_single_register(storage.register_temp_file(tf).await, "register");
        let fb = unwrap_storage(storage.take(id).await, "take");

        assert_eq!(fb.ext, Some(("image/png", "png")));
        assert_eq!(fb.dims, Some((3, 2)));

        CoverFileBuilder::from(fb)
            .build("png-kept")
            .await
            .unwrap_or_else(|_| panic!("build failed"));
        let got = read_all_bytes(&storage.reader, "covers/png-kept.png").await;
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn register_take_unsupported_image_is_converted_before_upload() {
        let backend = Arc::new(MemStorage::new());
        let storage = match StorageSystem::new(std::env::temp_dir().as_path(), backend).await {
            Ok(v) => v,
            Err(e) => panic!("storage init failed: {e}"),
        };

        let payload = bmp_bytes(4, 5);
        let mut tf = storage
            .new_temp_file()
            .await
            .unwrap_or_else(|_| panic!("tempfile create failed"));
        tf.write_all(&payload)
            .await
            .expect("write to tempfile should succeed");
        tf.sync_all().await.expect("sync should succeed");

        let id = unwrap_single_register(storage.register_temp_file(tf).await, "register");
        let fb = unwrap_storage(storage.take(id).await, "take");

        assert_eq!(fb.ext, Some(("image/jpeg", "jpeg")));
        assert_eq!(fb.dims, Some((4, 5)));

        CoverFileBuilder::from(fb)
            .build("bmp-converted")
            .await
            .unwrap_or_else(|_| panic!("build failed"));
        let got = read_all_bytes(&storage.reader, "covers/bmp-converted.jpeg").await;
        let kind = infer::get(&got).expect("converted object should be detected");
        assert_eq!(kind.mime_type(), "image/jpeg");
    }

    #[tokio::test]
    async fn register_chapter_container_returns_handle_array() {
        let backend = Arc::new(MemStorage::new());
        let storage = StorageSystem::new(std::env::temp_dir().as_path(), backend)
            .await
            .expect("storage init");

        let img1 = png_bytes(2, 2);
        let img2 = png_bytes(3, 4);
        let payload = make_chapter_container(&[img1.clone(), img2.clone()]);

        let mut tf = storage.new_temp_file().await.expect("temp");
        tf.write_all(&payload).await.expect("write");
        tf.sync_all().await.expect("sync");

        let registered = unwrap_storage(storage.register_temp_file(tf).await, "register chapter");
        let ids = match registered {
            RegisterTempResult::Chapter(ids) => ids,
            _ => panic!("expected chapter result"),
        };
        assert_eq!(ids.len(), 2);

        let fb1 = unwrap_storage(storage.take(ids[0].clone()).await, "take page 1");
        let fb2 = unwrap_storage(storage.take(ids[1].clone()).await, "take page 2");
        assert_eq!(fb1.dims, Some((2, 2)));
        assert_eq!(fb2.dims, Some((3, 4)));
    }

    #[tokio::test]
    async fn register_manga_container_keeps_image_order_and_metadata_handle() {
        let backend = Arc::new(MemStorage::new());
        let storage = StorageSystem::new(std::env::temp_dir().as_path(), backend)
            .await
            .expect("storage init");

        let img1 = png_bytes(10, 11);
        let img2 = png_bytes(12, 13);
        let img3 = png_bytes(14, 15);
        let metadata = MangaBundleMetadata {
            chapter_image_indexes: vec![vec![0, 1], vec![2]],
        };
        let payload = make_manga_container(&metadata, &[img1.clone(), img2.clone(), img3.clone()]);

        let mut tf = storage.new_temp_file().await.expect("temp");
        tf.write_all(&payload).await.expect("write");
        tf.sync_all().await.expect("sync");

        let registered = unwrap_storage(storage.register_temp_file(tf).await, "register manga");
        let manga = match registered {
            RegisterTempResult::Manga(v) => v,
            _ => panic!("expected manga result"),
        };
        assert_eq!(manga.images.len(), 3);
        assert_eq!(manga.chapter_image_indexes, metadata.chapter_image_indexes);

        let meta_fb = unwrap_storage(storage.take(manga.metadata.clone()).await, "take metadata");
        assert_eq!(meta_fb.ext, None);

        let fb1 = unwrap_storage(storage.take(manga.images[0].clone()).await, "take i1");
        let fb2 = unwrap_storage(storage.take(manga.images[1].clone()).await, "take i2");
        let fb3 = unwrap_storage(storage.take(manga.images[2].clone()).await, "take i3");
        assert_eq!(fb1.dims, Some((10, 11)));
        assert_eq!(fb2.dims, Some((12, 13)));
        assert_eq!(fb3.dims, Some((14, 15)));
    }
}
