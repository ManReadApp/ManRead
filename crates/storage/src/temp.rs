use std::{
    fs::File as StdFile,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::{Arc, Mutex as StdMutex},
};

use async_tempfile::TempFile;
use bytes::Bytes;
use futures_util::{stream, StreamExt as _};
use tokio::{
    fs::File,
    io::{AsyncReadExt as _, AsyncSeekExt as _},
};
use tokio_util::io::ReaderStream;

use crate::backends::ByteStream;

#[async_trait::async_trait]
pub(crate) trait TempData: Send + Sync {
    async fn len(&self) -> std::io::Result<u64>;
    async fn read_at(&self, offset: u64, len: usize) -> std::io::Result<Vec<u8>>;
    async fn open_stream(&self) -> std::io::Result<ByteStream>;
    fn slice(&self, offset: u64, len: u64) -> std::io::Result<Arc<dyn TempData>>;
    fn as_local_path(&self) -> Option<PathBuf>;

    async fn read_head(&self, len: usize) -> std::io::Result<Vec<u8>> {
        let total = self.len().await?;
        self.read_at(0, std::cmp::min(len, total as usize)).await
    }

    async fn read_all(&self) -> std::io::Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut stream = self.open_stream().await?;
        while let Some(chunk) = stream.next().await {
            out.extend_from_slice(&chunk?);
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub(crate) struct FileTempData {
    inner: Arc<FileTempInner>,
    offset: u64,
    len: u64,
}

struct FileTempInner {
    _temp: TempFile,
    file: StdMutex<StdFile>,
    path: PathBuf,
    full_len: u64,
}

impl FileTempData {
    pub(crate) async fn from_tempfile(mut tf: TempFile) -> std::io::Result<Self> {
        use tokio::io::AsyncWriteExt as _;

        tf.flush().await?;
        tf.sync_all().await?;
        let full_len = tf.metadata().await?.len();
        Ok(Self {
            inner: Arc::new(FileTempInner {
                file: StdMutex::new(StdFile::open(tf.file_path())?),
                path: tf.file_path().to_path_buf(),
                _temp: tf,
                full_len,
            }),
            offset: 0,
            len: full_len,
        })
    }
}

#[async_trait::async_trait]
impl TempData for FileTempData {
    async fn len(&self) -> std::io::Result<u64> {
        Ok(self.len)
    }

    async fn read_at(&self, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        if offset > self.len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "offset out of range",
            ));
        }
        let abs = self.offset + offset;
        let max = (self.len - offset) as usize;
        let read_len = std::cmp::min(len, max);
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner
                .file
                .lock()
                .map_err(|_| std::io::Error::other("temp file lock poisoned"))?;
            guard.seek(SeekFrom::Start(abs))?;
            let mut out = vec![0u8; read_len];
            guard.read_exact(&mut out)?;
            Ok(out)
        })
        .await
        .map_err(|e| std::io::Error::other(format!("temp read task failed: {e}")))?
    }

    async fn open_stream(&self) -> std::io::Result<ByteStream> {
        let mut file = File::open(&self.inner.path).await?;
        file.seek(std::io::SeekFrom::Start(self.offset)).await?;
        let reader = file.take(self.len);
        let stream = ReaderStream::new(reader).boxed();
        Ok(Box::pin(stream))
    }

    fn slice(&self, offset: u64, len: u64) -> std::io::Result<Arc<dyn TempData>> {
        if offset > self.len || len > (self.len - offset) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "slice out of range",
            ));
        }
        Ok(Arc::new(Self {
            inner: self.inner.clone(),
            offset: self.offset + offset,
            len,
        }))
    }

    fn as_local_path(&self) -> Option<PathBuf> {
        if self.offset == 0 && self.len == self.inner.full_len {
            Some(self.inner.path.clone())
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub(crate) struct MemoryTempData {
    data: Bytes,
    offset: usize,
    len: usize,
}

impl MemoryTempData {
    pub(crate) fn from_bytes(data: Vec<u8>) -> Self {
        let bytes = Bytes::from(data);
        let len = bytes.len();
        Self {
            data: bytes,
            offset: 0,
            len,
        }
    }
}

#[async_trait::async_trait]
impl TempData for MemoryTempData {
    async fn len(&self) -> std::io::Result<u64> {
        Ok(self.len as u64)
    }

    async fn read_at(&self, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let offset = offset as usize;
        if offset > self.len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "offset out of range",
            ));
        }
        let max = self.len - offset;
        let read_len = std::cmp::min(len, max);
        let start = self.offset + offset;
        let end = start + read_len;
        Ok(self.data.slice(start..end).to_vec())
    }

    async fn open_stream(&self) -> std::io::Result<ByteStream> {
        let start = self.offset;
        let end = start + self.len;
        let chunk = self.data.slice(start..end);
        let stream = stream::once(async move { Ok::<Bytes, std::io::Error>(chunk) });
        Ok(Box::pin(stream))
    }

    fn slice(&self, offset: u64, len: u64) -> std::io::Result<Arc<dyn TempData>> {
        let offset = offset as usize;
        let len = len as usize;
        if offset > self.len || len > (self.len - offset) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "slice out of range",
            ));
        }
        Ok(Arc::new(Self {
            data: self.data.clone(),
            offset: self.offset + offset,
            len,
        }))
    }

    fn as_local_path(&self) -> Option<PathBuf> {
        None
    }
}
