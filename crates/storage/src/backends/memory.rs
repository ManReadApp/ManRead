use bytes::{Bytes, BytesMut};
use futures_util::{stream, TryStreamExt as _};
use std::{collections::HashMap, io};
use tokio::sync::RwLock;

use crate::backends::{ByteStream, Object, Options, StorageReader, StorageWriter};

pub struct MemStorage {
    inner: RwLock<HashMap<String, Bytes>>,
}

impl MemStorage {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    async fn put(&self, key: impl Into<String>, data: Bytes) {
        self.inner.write().await.insert(key.into(), data);
    }
}

#[async_trait::async_trait]
impl StorageWriter for MemStorage {
    async fn write(
        &self,
        key: &str,
        _: Option<Options>,
        mut stream: ByteStream,
    ) -> Result<(), io::Error> {
        let mut buf = BytesMut::new();

        while let Some(chunk) = stream.try_next().await? {
            buf.extend_from_slice(&chunk);
        }

        self.put(key.to_string(), buf.freeze()).await;
        Ok(())
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        let mut map = self.inner.write().await;

        let data = map.remove(orig_key).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("key not found: {orig_key}"),
            )
        })?;

        map.insert(target_key.to_string(), data);
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageReader for MemStorage {
    async fn get(&self, key: &str, _: Option<Options>) -> Result<Object, std::io::Error> {
        let map = self.inner.read().await;
        let data = map
            .get(key)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))?;

        let len = data.len() as u64;

        let stream = stream::once(async move { Ok::<Bytes, std::io::Error>(data) });
        let stream: ByteStream = Box::pin(stream);

        Ok(Object {
            stream,
            content_length: Some(len),
            content_type: None,
            last_modified: None,
            etag: None,
        })
    }
}
