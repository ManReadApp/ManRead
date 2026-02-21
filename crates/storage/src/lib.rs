mod backends;
mod builders;
mod error;
mod temp;
mod workers;

pub use backends::CacheBackend;
pub use backends::ContentLengthStorage;
pub use backends::DelayStorage;
#[cfg(feature = "disk")]
pub use backends::DiskStorage;
#[cfg(feature = "encode")]
pub use backends::EncryptedStorage;
pub use backends::InMemoryKeyValueStore;
pub use backends::KeyValueStore;
pub use backends::MemStorage;
pub use backends::Object;
pub use backends::Options;
pub use backends::StorageReader;
pub use backends::StorageWriter;
#[cfg(feature = "s3")]
pub use backends::{S3Storage, S3StorageOptions, S3UploadAcl};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub use async_tempfile::TempFile;

pub use builders::{
    ArtFileBuilder, CoverFileBuilder, FileBuilder, MangaPageFileBuilder, UserBannerBuilder,
    UserCoverFileBuilder,
};
pub use error::StorageError;
use futures_util::{FutureExt as _, StreamExt as _, TryStreamExt as _};
use rand::prelude::IndexedRandom;
use tokio::{
    fs::File,
    sync::{watch, Mutex, Semaphore},
};

use crate::{
    error::{ProcessingError, StorageResult},
    temp::{FileTempData, TempData},
    workers::{
        containers::{ContainerPayload, ContainerWorker, MagicContainerWorker},
        media::{file_to_bytestream, DefaultMediaWorker, MediaWorker},
    },
};

#[cfg(test)]
pub(crate) use workers::containers::{CHAPTER_MAGIC, MANGA_MAGIC};

pub struct StorageSystem {
    files: Arc<Mutex<HashMap<String, StoredFile>>>,
    transcode_sem: Arc<Semaphore>,
    inflight_sem: Arc<Semaphore>,
    writer: Arc<dyn StorageWriter + Send + Sync>,
    pub reader: Arc<dyn StorageReader + Send + Sync>,
    path: PathBuf,
    container_worker: Arc<dyn ContainerWorker + Send + Sync>,
    media_worker: Arc<dyn MediaWorker + Send + Sync>,
}

struct StoredFile {
    ext: Option<(&'static str, &'static str)>,
    dims: Option<(u32, u32)>,
    state: EntryState,
}

enum EntryState {
    Processing { state_tx: watch::Sender<()> },
    Uploaded { handle: String },
    Failed { error: ProcessingError },
}

impl StoredFile {
    fn new_processing(
        ext: Option<(&'static str, &'static str)>,
        state_tx: watch::Sender<()>,
    ) -> Self {
        Self {
            ext,
            dims: None,
            state: EntryState::Processing { state_tx },
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

impl StorageSystem {
    fn inflight_limit(transcode_limit: usize) -> usize {
        transcode_limit.max(1).saturating_mul(4)
    }

    fn build_with_components(
        path: &Path,
        reader: Arc<dyn StorageReader + Send + Sync>,
        writer: Arc<dyn StorageWriter + Send + Sync>,
        container_worker: Arc<dyn ContainerWorker + Send + Sync>,
        media_worker: Arc<dyn MediaWorker + Send + Sync>,
        transcode_limit: usize,
    ) -> Self {
        StorageSystem {
            reader,
            writer,
            path: path.to_path_buf(),
            files: Arc::default(),
            transcode_sem: Arc::new(Semaphore::new(transcode_limit)),
            inflight_sem: Arc::new(Semaphore::new(Self::inflight_limit(transcode_limit))),
            container_worker,
            media_worker,
        }
    }

    pub async fn new_temp_file(&self) -> StorageResult<TempFile> {
        Ok(TempFile::new().await?)
    }

    async fn register_many_files(
        &self,
        files: Vec<Arc<dyn TempData>>,
    ) -> StorageResult<Vec<FileId>> {
        let stream = futures_util::stream::iter(
            files
                .into_iter()
                .map(|file| async move { self.register_single_temp_file(file).await }),
        );

        stream.buffered(8).try_collect().await
    }

    async fn register_single_temp_file(&self, source: Arc<dyn TempData>) -> StorageResult<FileId> {
        let id = uuid::Uuid::new_v4().to_string();
        let inflight_permit = self
            .inflight_sem
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| {
                StorageError::Io(std::io::Error::other("upload queue semaphore closed"))
            })?;
        let (state_tx, _) = watch::channel(());

        {
            let mut map = self.files.lock().await;
            map.insert(
                id.clone(),
                StoredFile::new_processing(None, state_tx.clone()),
            );
        }

        let files = self.files.clone();
        let sem = self.transcode_sem.clone();
        let writer = self.writer.clone();
        let media_worker = self.media_worker.clone();
        let id2 = id.clone();

        tokio::spawn(async move {
            let _inflight_permit = inflight_permit;
            let result =
                std::panic::AssertUnwindSafe(media_worker.process_and_upload(source, writer, sem))
                    .catch_unwind()
                    .await;

            let updated = {
                let mut map = files.lock().await;
                if let Some(entry) = map.get_mut(&id2) {
                    match result {
                        Ok(Ok(upload)) => {
                            entry.mark_uploaded(upload.handle, upload.dims, upload.ext);
                        }
                        Ok(Err(error)) => {
                            entry.mark_failed(error);
                        }
                        Err(_) => {
                            entry.mark_failed(ProcessingError::BackgroundTaskPanic);
                        }
                    }

                    true
                } else {
                    false
                }
            };

            if updated {
                let _ = state_tx.send(());
            }
        });

        Ok(FileId::new(id))
    }

    pub async fn register_temp_file(&self, tf: TempFile) -> StorageResult<RegisterTempResult> {
        let source: Arc<dyn TempData> = Arc::new(FileTempData::from_tempfile(tf).await?);
        match self.container_worker.extract_payload(source).await? {
            ContainerPayload::SingleFile(tf) => self
                .register_single_temp_file(tf)
                .await
                .map(RegisterTempResult::File),
            ContainerPayload::Chapter(images) => self
                .register_many_files(images)
                .await
                .map(RegisterTempResult::Chapter),
            ContainerPayload::Manga {
                metadata,
                chapter_image_indexes,
                images,
            } => {
                let metadata_id = self.register_single_temp_file(metadata).await?;
                let image_ids = self.register_many_files(images).await?;

                Ok(RegisterTempResult::Manga(RegisteredMangaTemp {
                    metadata: metadata_id,
                    images: image_ids,
                    chapter_image_indexes,
                }))
            }
        }
    }

    pub async fn take_bytes(&self, id: FileId) -> StorageResult<Vec<u8>> {
        let mut file = self.take(id).await?;
        file.allowed_drop = true;
        let key = file.temp_id.clone();
        drop(file);

        let object = self.reader.get(&key, &Default::default()).await?;
        let bytes = object
            .stream
            .try_fold(Vec::new(), |mut acc, chunk| async move {
                acc.extend_from_slice(&chunk);
                Ok(acc)
            })
            .await?;
        self.writer.delete(&key).await?;
        Ok(bytes)
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
                        let Some(entry) = map.remove(id.inner_ref()) else {
                            return Err(StorageError::HandleNotFound);
                        };
                        return Ok(FileBuilder {
                            dims: entry.dims,
                            ext: entry.ext,
                            temp_id: handle,
                            target_id: PathBuf::new(),
                            allowed_drop: false,
                            writer: self.writer.clone(),
                        });
                    }
                    EntryState::Processing { state_tx } => Some(state_tx.subscribe()),
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

            if let Some(mut rx) = wait_on {
                let _ = rx.changed().await;
            }
        }
    }

    pub async fn delete_key(&self, key: &str) -> StorageResult<()> {
        self.writer.delete(key).await?;
        Ok(())
    }

    pub async fn get_user_cover(&self, id: Option<FileId>) -> StorageResult<UserCoverFileBuilder> {
        let item = match id {
            Some(id) => self.take(id).await.map(UserCoverFileBuilder::from),
            None => {
                let p = self.path.join("cover_templates");
                let ri = get_random_image(&p)
                    .await
                    .ok_or(StorageError::NoDefaultImageAvailable)?;
                let f = File::open(&ri).await?;
                let id = PathBuf::from(format!("temp/{}", uuid::Uuid::new_v4()));
                let id = id.to_string_lossy();
                self.writer.write(&id, file_to_bytestream(f)).await?;
                Ok(UserCoverFileBuilder::from(FileBuilder {
                    dims: None,
                    allowed_drop: false,
                    temp_id: id.to_string(),
                    ext: {
                        let mut bytes = Vec::new();
                        let file = File::open(&ri).await?;
                        use tokio::io::AsyncReadExt as _;
                        file.take(8192).read_to_end(&mut bytes).await?;
                        infer::get(&bytes).map(|k| {
                            if k.extension() == "jpg" {
                                (k.mime_type(), "jpeg")
                            } else {
                                (k.mime_type(), k.extension())
                            }
                        })
                    },
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
        let reader: Arc<dyn StorageReader + Send + Sync> = rw.clone();
        let writer: Arc<dyn StorageWriter + Send + Sync> = rw;
        Ok(Self::build_with_components(
            path,
            reader,
            writer,
            Arc::new(MagicContainerWorker),
            Arc::new(DefaultMediaWorker),
            5,
        ))
    }

    pub async fn new_with_rw(
        path: &Path,
        reader: Arc<dyn StorageReader + Send + Sync>,
        writer: Arc<dyn StorageWriter + Send + Sync>,
        transcode_limit: usize,
    ) -> std::io::Result<Self> {
        Ok(Self::build_with_components(
            path,
            reader,
            writer,
            Arc::new(MagicContainerWorker),
            Arc::new(DefaultMediaWorker),
            transcode_limit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Cursor,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use image::{DynamicImage, ImageFormat};
    use tokio::io::AsyncWriteExt as _;

    use export::manga::{
        Chapter as BundleChapter, ChapterVersion as BundleChapterVersion, MangaBundleMetadata,
    };

    use crate::{
        backends::StorageWriter,
        error::ProcessingError,
        temp::{MemoryTempData, TempData},
        workers::{
            containers::MagicContainerWorker,
            media::{MediaWorker, PreparedUpload},
        },
        CoverFileBuilder, FileId, MemStorage, RegisterTempResult, StorageError, StorageSystem,
        CHAPTER_MAGIC, MANGA_MAGIC,
    };

    async fn read_all_bytes(
        reader: &Arc<dyn crate::backends::StorageReader + Send + Sync>,
        key: &str,
    ) -> Vec<u8> {
        use futures_util::StreamExt as _;

        let mut out = Vec::new();
        let mut stream = reader
            .get(key, &Default::default())
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
        let metadata_bytes = export::to_bytes(metadata);
        let mut out = Vec::new();
        out.extend_from_slice(MANGA_MAGIC);
        out.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(metadata_bytes.as_ref());
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
            .build("plain-roundtrip", 0)
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
            .build("png-kept", 0)
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
            .build("bmp-converted", 0)
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
            titles: Default::default(),
            kind: "manga".to_owned(),
            description: None,
            tags: vec![],
            status: 0,
            visibility: 0,
            uploader: "uploader".to_owned(),
            artists: vec![],
            authors: vec![],
            publishers: vec![],
            sources: vec![],
            scraper: vec![],
            volumes: vec![],
            cover_image_indexes: vec![],
            art_image_indexes: vec![],
            chapters: vec![
                BundleChapter {
                    titles: vec!["chapter-1".to_owned()],
                    chapter: 1.0,
                    tags: vec![],
                    sources: vec![],
                    release_date: None,
                    versions: vec![BundleChapterVersion {
                        version: "version-1".to_owned(),
                        image_indexes: vec![0, 1],
                        link: None,
                    }],
                },
                BundleChapter {
                    titles: vec!["chapter-2".to_owned()],
                    chapter: 2.0,
                    tags: vec![],
                    sources: vec![],
                    release_date: None,
                    versions: vec![BundleChapterVersion {
                        version: "version-1".to_owned(),
                        image_indexes: vec![2],
                        link: None,
                    }],
                },
            ],
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
        assert_eq!(
            manga.chapter_image_indexes,
            metadata
                .chapters
                .iter()
                .flat_map(|chapter| chapter.versions.iter().map(|v| v.image_indexes.clone()))
                .collect::<Vec<_>>()
        );

        let meta_fb = unwrap_storage(storage.take(manga.metadata.clone()).await, "take metadata");
        assert_eq!(meta_fb.ext, None);

        let fb1 = unwrap_storage(storage.take(manga.images[0].clone()).await, "take i1");
        let fb2 = unwrap_storage(storage.take(manga.images[1].clone()).await, "take i2");
        let fb3 = unwrap_storage(storage.take(manga.images[2].clone()).await, "take i3");
        assert_eq!(fb1.dims, Some((10, 11)));
        assert_eq!(fb2.dims, Some((12, 13)));
        assert_eq!(fb3.dims, Some((14, 15)));
    }

    struct PanicMediaWorker;

    #[async_trait::async_trait]
    impl MediaWorker for PanicMediaWorker {
        async fn detect_ext(
            &self,
            _source: &Arc<dyn TempData>,
        ) -> Option<(&'static str, &'static str)> {
            None
        }

        async fn process_and_upload(
            &self,
            _source: Arc<dyn TempData>,
            _writer: Arc<dyn StorageWriter + Send + Sync>,
            _transcode_sem: Arc<tokio::sync::Semaphore>,
        ) -> Result<PreparedUpload, ProcessingError> {
            panic!("intentional panic for test")
        }
    }

    struct SlowCountingMediaWorker {
        in_flight: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
        delay: Duration,
    }

    impl SlowCountingMediaWorker {
        fn record_peak(&self, candidate: usize) {
            let mut current = self.peak.load(Ordering::SeqCst);
            while candidate > current {
                match self.peak.compare_exchange(
                    current,
                    candidate,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => return,
                    Err(next) => current = next,
                }
            }
        }
    }

    #[async_trait::async_trait]
    impl MediaWorker for SlowCountingMediaWorker {
        async fn detect_ext(
            &self,
            _source: &Arc<dyn TempData>,
        ) -> Option<(&'static str, &'static str)> {
            None
        }

        async fn process_and_upload(
            &self,
            _source: Arc<dyn TempData>,
            _writer: Arc<dyn StorageWriter + Send + Sync>,
            _transcode_sem: Arc<tokio::sync::Semaphore>,
        ) -> Result<PreparedUpload, ProcessingError> {
            let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.record_peak(now);
            tokio::time::sleep(self.delay).await;
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
            Ok(PreparedUpload {
                handle: format!("temp/{}", uuid::Uuid::new_v4()),
                dims: None,
                ext: None,
            })
        }
    }

    #[tokio::test]
    async fn register_take_returns_error_when_worker_panics() {
        let backend = Arc::new(MemStorage::new());
        let reader: Arc<dyn crate::backends::StorageReader + Send + Sync> = backend.clone();
        let writer: Arc<dyn crate::backends::StorageWriter + Send + Sync> = backend;
        let storage = StorageSystem::build_with_components(
            std::env::temp_dir().as_path(),
            reader,
            writer,
            Arc::new(MagicContainerWorker),
            Arc::new(PanicMediaWorker),
            1,
        );

        let payload = b"panic-path".to_vec();
        let mut tf = storage
            .new_temp_file()
            .await
            .expect("tempfile create failed");
        tf.write_all(&payload)
            .await
            .expect("write to tempfile should succeed");
        tf.sync_all().await.expect("sync should succeed");

        let id = unwrap_single_register(storage.register_temp_file(tf).await, "register");
        let err = match storage.take(id).await {
            Ok(_) => panic!("take should fail"),
            Err(err) => err,
        };
        match err {
            StorageError::Processing(ProcessingError::BackgroundTaskPanic) => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn register_many_limits_inflight_background_workers() {
        let backend = Arc::new(MemStorage::new());
        let reader: Arc<dyn crate::backends::StorageReader + Send + Sync> = backend.clone();
        let writer: Arc<dyn crate::backends::StorageWriter + Send + Sync> = backend;
        let in_flight = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let storage = StorageSystem::build_with_components(
            std::env::temp_dir().as_path(),
            reader,
            writer,
            Arc::new(MagicContainerWorker),
            Arc::new(SlowCountingMediaWorker {
                in_flight: in_flight.clone(),
                peak: peak.clone(),
                delay: Duration::from_millis(60),
            }),
            1,
        );

        let files: Vec<Arc<dyn TempData>> = (0..20)
            .map(|i| Arc::new(MemoryTempData::from_bytes(vec![i as u8])) as Arc<dyn TempData>)
            .collect();

        let ids = storage
            .register_many_files(files)
            .await
            .expect("register_many should succeed");
        for id in ids {
            storage.take(id).await.expect("take should succeed");
        }

        assert_eq!(in_flight.load(Ordering::SeqCst), 0);
        assert!(
            peak.load(Ordering::SeqCst) <= StorageSystem::inflight_limit(1),
            "in-flight workers should be bounded by queue capacity"
        );
    }

    #[tokio::test]
    async fn build_rejects_unsafe_target_paths() {
        let backend = Arc::new(MemStorage::new());
        let storage = StorageSystem::new(std::env::temp_dir().as_path(), backend)
            .await
            .expect("storage init");

        let payload = b"unsafe-path".to_vec();
        let mut tf = storage
            .new_temp_file()
            .await
            .expect("tempfile create failed");
        tf.write_all(&payload)
            .await
            .expect("write to tempfile should succeed");
        tf.sync_all().await.expect("sync should succeed");

        let id = unwrap_single_register(storage.register_temp_file(tf).await, "register");
        let fb = unwrap_storage(storage.take(id).await, "take");
        let err = CoverFileBuilder::from(fb)
            .build("../escape", 0)
            .await
            .expect_err("unsafe target should be rejected");

        match err {
            StorageError::Io(ioe) => assert_eq!(ioe.kind(), std::io::ErrorKind::InvalidInput),
            other => panic!("unexpected error: {other}"),
        }
    }
}
