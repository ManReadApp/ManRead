use std::{
    io,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use async_tempfile::TempFile;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt as _};
use tokio_util::io::ReaderStream;

use crate::backends::{ByteStream, Object, Options, StorageReader, StorageWriter};

pub struct DiskStorage {
    root: PathBuf,
}

impl DiskStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn key_path(&self, key: &str) -> Result<PathBuf, io::Error> {
        let rel = Path::new(key);
        if rel.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "storage key cannot be empty",
            ));
        }

        for comp in rel.components() {
            match comp {
                Component::Normal(_) => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "storage key must be a safe relative path",
                    ));
                }
            }
        }

        Ok(self.root.join(rel))
    }

    async fn rename_replace(src: &Path, dst: &Path) -> Result<(), io::Error> {
        match tokio::fs::rename(src, dst).await {
            Ok(()) => Ok(()),
            Err(err) => {
                let dst_exists = tokio::fs::metadata(dst).await.is_ok();
                let can_retry = matches!(
                    err.kind(),
                    io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
                );

                if !dst_exists || !can_retry {
                    return Err(err);
                }

                tokio::fs::remove_file(dst).await?;
                tokio::fs::rename(src, dst).await
            }
        }
    }
}

#[async_trait::async_trait]
impl StorageWriter for DiskStorage {
    async fn write(&self, key: &str, stream: ByteStream) -> Result<(), io::Error> {
        let target = self.key_path(key)?;
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut stream = Box::pin(stream);

        let parent = target.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "storage key must contain a parent directory",
            )
        })?;

        let mut file = TempFile::new_in(parent)
            .await
            .map_err(|e| io::Error::other(format!("tempfile create failed: {e}")))?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(chunk.as_ref()).await?;
        }

        file.flush().await?;
        file.sync_all().await?;

        Self::rename_replace(file.file_path().as_path(), &target).await?;
        file.drop_async().await;
        Ok(())
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        let src = self.key_path(orig_key)?;
        let dst = self.key_path(target_key)?;
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Self::rename_replace(&src, &dst).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageReader for DiskStorage {
    async fn get(&self, key: &str, _: &Options) -> Result<Object, std::io::Error> {
        let path = self.key_path(key)?;
        let file = File::open(&path).await?;
        let meta = file.metadata().await.ok();
        let mut lm = meta
            .as_ref()
            .map(|v| v.modified().ok())
            .flatten()
            .map(|v| v.duration_since(UNIX_EPOCH).ok())
            .flatten();
        if let Some(secs) = lm.map(|v| v.as_secs()) {
            if secs >= 253_402_300_800 {
                lm = None;
            }
        }

        let len = meta.map(|m| m.len());

        let stream = ReaderStream::new(file).map(|r| r.map(Bytes::from));
        let stream: ByteStream = Box::pin(stream);

        Ok(Object {
            stream,
            content_length: len,
            content_type: None,
            etag: None,
            last_modified: lm,
        })
    }
}
