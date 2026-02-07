mod backends;
mod builders;
mod error;

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
    io::{self, AsyncReadExt as _, AsyncWriteExt},
    sync::{Mutex, Notify, Semaphore},
};
use tokio_util::io::ReaderStream;

use crate::{
    backends::{ByteStream, StorageReader, StorageWriter},
    error::StorageResult,
};

pub struct StorageSystem {
    files: Arc<Mutex<HashMap<String, StoredFile>>>,
    transcode_sem: Arc<Semaphore>,
    writer: Arc<dyn StorageWriter + Send + Sync>,
    pub reader: Arc<dyn StorageReader + Send + Sync>,
    path: PathBuf,
}

struct StoredFile {
    tf: TempFile,
    ext: Option<(&'static str, &'static str)>,
    dims: Option<(u32, u32)>,
    state: EntryState,
}

enum EntryState {
    Ready,
    Processing(Arc<Notify>),
}

pub trait FileBuilderExt {
    fn width(&self) -> Option<u32>;
    fn height(&self) -> Option<u32>;
    fn ext(&self) -> StorageResult<&str>;
}

pub fn file_to_bytestream(file: File) -> ByteStream {
    ReaderStream::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e)) // tokio_util error -> io::Error
        .boxed()
}

async fn get_random_image(folder: &Path) -> Option<PathBuf> {
    let extensions = ["png", "gif", "jpeg", "jpg", "qoi"];

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

    pub async fn register_temp_file(&self, mut tf: TempFile) -> StorageResult<FileId> {
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
        let state = if is_image && !allowed {
            EntryState::Processing(notify.clone())
        } else {
            EntryState::Ready
        };

        {
            let mut map = self.files.lock().await;
            map.insert(
                id.clone(),
                StoredFile {
                    tf,
                    ext,
                    dims: None,
                    state,
                },
            );
        }

        if is_image {
            if !allowed {
                let files = self.files.clone();
                let sem = self.transcode_sem.clone();
                let id2 = id.clone();

                tokio::spawn(async move {
                    let _permit = match sem.acquire().await {
                        Ok(p) => p,
                        Err(_) => return,
                    };

                    let buffer = {
                        let mut buffer = vec![];

                        let mut map = files.lock().await;
                        let Some(entry) = map.get_mut(&id2) else {
                            return;
                        };
                        entry.tf.read_to_end(&mut buffer).await.unwrap();
                        buffer
                    };
                    let res: Result<
                        Result<(Vec<u8>, (u32, u32), (&str, &str)), image::ImageError>,
                        tokio::task::JoinError,
                    > = tokio::task::spawn_blocking(move || {
                        let img = image::load_from_memory(&buffer)?;
                        let mut c = Cursor::new(vec![]);
                        let size = (img.width(), img.height());
                        let enc = image::codecs::jpeg::JpegEncoder::new(&mut c);
                        img.write_with_encoder(enc)?;
                        Ok((c.into_inner(), size, ("image/jpeg", "jpeg")))
                    })
                    .await;

                    let notify_to_wake = {
                        let mut map = files.lock().await;
                        let Some(entry) = map.get_mut(&id2) else {
                            return;
                        };

                        let n = match &entry.state {
                            EntryState::Processing(n) => n.clone(),
                            _ => Arc::new(Notify::new()),
                        };

                        if let Ok(Ok((data, dims, new_ext))) = res {
                            entry.dims = Some(dims);
                            let mut tf = TempFile::new().await.unwrap();
                            tf.write_all(&data).await.unwrap();
                            tf.flush().await.unwrap();
                            tf.sync_all().await.unwrap();
                            entry.tf = tf;
                            entry.ext = Some(new_ext);
                        }
                        entry.state = EntryState::Ready;

                        n
                    };

                    notify_to_wake.notify_waiters();
                });
            } else {
                let f = self
                    .files
                    .lock()
                    .await
                    .get(&id)
                    .unwrap()
                    .tf
                    .file_path()
                    .to_owned();
                let dims = tokio::task::spawn_blocking(move || image::image_dimensions(&f).ok())
                    .await
                    .ok()
                    .flatten();
                self.files.lock().await.get_mut(&id).unwrap().dims = dims;
            }
        }

        Ok(FileId::new(id))
    }

    pub async fn take(&self, id: FileId) -> StorageResult<FileBuilder> {
        loop {
            let wait_on = {
                let mut map = self.files.lock().await;
                let entry = map
                    .get(id.inner_ref())
                    .ok_or(StorageError::HandleNotFound)?;

                match &entry.state {
                    EntryState::Ready => {
                        let entry = map.remove(id.inner_ref()).unwrap();
                        return Ok(FileBuilder {
                            dims: entry.dims,
                            ext: entry.ext,
                            temp_id: RealFile::TempFile(entry.tf),
                            target_id: PathBuf::new(),
                            allowed_drop: false,
                            writer: self.writer.clone(),
                        });
                    }
                    EntryState::Processing(n) => Some(n.clone()),
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
                self.writer.write(&id, options, file_to_bytestream(f));
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
