use std::{io, time::UNIX_EPOCH};

use async_tempfile::TempFile;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt as _};
use tokio_util::io::ReaderStream;

use crate::backends::{ByteStream, Object, Options, StorageReader, StorageWriter};

pub struct DiskStorage {
    root: std::path::PathBuf,
}

impl DiskStorage {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[async_trait::async_trait]
impl StorageWriter for DiskStorage {
    async fn write(
        &self,
        key: &str,
        _: Option<Options>,
        stream: ByteStream,
    ) -> Result<(), io::Error> {
        let target = self.root.join(key);
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut stream = Box::pin(stream);

        let mut file = TempFile::new().await.unwrap();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(chunk.as_ref()).await?;
        }

        file.flush().await?;
        file.sync_all().await?;

        if tokio::fs::metadata(&target).await.is_ok() {
            let _ = tokio::fs::remove_file(&target).await;
        }

        tokio::fs::rename(&file.file_path(), &target).await?;
        file.drop_async().await;
        Ok(())
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        let src = self.root.join(orig_key);
        let dst = self.root.join(target_key);
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if tokio::fs::metadata(&dst).await.is_ok() {
            let _ = tokio::fs::remove_file(&dst).await;
        }

        tokio::fs::rename(&src, &dst).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageReader for DiskStorage {
    async fn get(&self, key: &str, _: Option<Options>) -> Result<Object, std::io::Error> {
        let path = self.root.join(key);
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
