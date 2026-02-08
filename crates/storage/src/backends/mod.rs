#[cfg(feature = "encode")]
mod aes_gcm;
mod delay;
#[cfg(feature = "disk")]
mod disk;
mod memory;

#[cfg(feature = "encode")]
pub use aes_gcm::EncryptedStorage;
pub use delay::DelayStorage;
#[cfg(feature = "disk")]
pub use disk::DiskStorage;
pub use memory::MemStorage;

use std::{pin::Pin, time::Duration};

use bytes::Bytes;
use futures_core::Stream;

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub struct Object {
    pub stream: ByteStream,
    pub content_length: Option<u64>,
    pub content_type: Option<mime::Mime>,
    pub etag: Option<String>,
    pub last_modified: Option<Duration>,
}

#[derive(Clone)]
pub struct Options {
    key: [u8; 32],
    nonce: [u8; 12],
    counter: u32,
    aad: Vec<u8>,
}

impl Options {
    pub fn new(key: [u8; 32], nonce: [u8; 12], counter: u32, aad: Vec<u8>) -> Self {
        Self {
            key,
            nonce,
            counter,
            aad,
        }
    }
}
#[async_trait::async_trait]
pub trait StorageReader: Send + Sync + 'static {
    /// reads file as stream
    async fn get(&self, key: &str, options: Option<Options>) -> Result<Object, std::io::Error>;
}

#[async_trait::async_trait]
pub trait StorageWriter: Send + Sync + 'static {
    /// add new object
    async fn write(
        &self,
        key: &str,
        options: Option<Options>,
        stream: ByteStream,
    ) -> Result<(), std::io::Error>;

    /// already uploaded under a different key(old key gets removed and added under new key)
    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), std::io::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};
    use futures_util::{stream, StreamExt as _};
    use std::time::{Duration, Instant};

    fn stream_from_parts(parts: Vec<Vec<u8>>) -> ByteStream {
        let chunks = parts
            .into_iter()
            .map(|part| Ok::<Bytes, std::io::Error>(Bytes::from(part)));
        Box::pin(stream::iter(chunks))
    }

    async fn read_all(mut stream: ByteStream) -> Result<Vec<u8>, std::io::Error> {
        let mut out = Vec::new();
        while let Some(chunk) = stream.next().await {
            out.extend_from_slice(&chunk?);
        }
        Ok(out)
    }

    async fn assert_roundtrip<T>(
        storage: &T,
        key: &str,
        payload: &[u8],
        options: Option<Options>,
    ) -> Result<(), std::io::Error>
    where
        T: StorageReader + StorageWriter,
    {
        storage
            .write(
                key,
                options.clone(),
                stream_from_parts(vec![
                    payload[..8].to_vec(),
                    payload[8..17].to_vec(),
                    payload[17..].to_vec(),
                ]),
            )
            .await?;

        let obj = storage.get(key, options).await?;
        let got = read_all(obj.stream).await?;
        assert_eq!(got, payload);
        Ok(())
    }

    #[tokio::test]
    async fn roundtrip_mem_no_aes() -> Result<(), std::io::Error> {
        let storage = MemStorage::new();
        let payload = b"plain-memory-roundtrip-with-multiple-chunks";
        assert_roundtrip(&storage, "plain/mem", payload, None).await
    }

    #[tokio::test]
    async fn roundtrip_mem_with_delay_no_aes() -> Result<(), std::io::Error> {
        let storage = DelayStorage::new(MemStorage::new(), Duration::from_millis(20));
        let payload = b"delayed-memory-roundtrip-with-multiple-chunks";

        let started = Instant::now();
        assert_roundtrip(&storage, "plain/delayed", payload, None).await?;
        assert!(
            started.elapsed() >= Duration::from_millis(35),
            "delay wrapper should apply delay to write and read paths"
        );
        Ok(())
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_aes() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(MemStorage::new());
        let payload = b"encrypted-memory-roundtrip-with-multiple-chunks";
        let opts = Options::new([7u8; 32], [9u8; 12], 0, b"roundtrip-aad".to_vec());

        assert_roundtrip(&storage, "enc/mem", payload, Some(opts)).await
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_delay_and_aes_and_rename() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(DelayStorage::new(
            MemStorage::new(),
            Duration::from_millis(10),
        ));
        let payload = b"encrypted-delayed-memory-roundtrip-with-multiple-chunks";
        let opts = Options::new([3u8; 32], [5u8; 12], 0, b"rename-aad".to_vec());

        storage
            .write(
                "tmp/key",
                Some(opts.clone()),
                stream_from_parts(vec![
                    payload[..13].to_vec(),
                    payload[13..29].to_vec(),
                    payload[29..].to_vec(),
                ]),
            )
            .await?;
        storage.rename("tmp/key", "final/key").await?;

        let obj = storage.get("final/key", Some(opts)).await?;
        let got = read_all(obj.stream).await?;
        assert_eq!(got.as_slice(), payload);
        Ok(())
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_aes_nonzero_counter() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(MemStorage::new());
        let payload = b"encrypted-memory-roundtrip-with-nonzero-counter";
        let opts = Options::new([11u8; 32], [13u8; 12], 42, b"counter-aad".to_vec());

        assert_roundtrip(&storage, "enc/mem/nonzero-counter", payload, Some(opts)).await
    }

    #[cfg(feature = "encode")]
    struct ChunkingMemStorage {
        inner: tokio::sync::Mutex<std::collections::HashMap<String, Bytes>>,
        chunk_len: usize,
    }

    #[cfg(feature = "encode")]
    impl ChunkingMemStorage {
        fn new(chunk_len: usize) -> Self {
            Self {
                inner: tokio::sync::Mutex::new(std::collections::HashMap::new()),
                chunk_len,
            }
        }
    }

    #[cfg(feature = "encode")]
    #[async_trait::async_trait]
    impl StorageWriter for ChunkingMemStorage {
        async fn write(
            &self,
            key: &str,
            _: Option<Options>,
            mut stream: ByteStream,
        ) -> Result<(), std::io::Error> {
            let mut out = BytesMut::new();
            while let Some(chunk) = stream.next().await {
                out.extend_from_slice(&chunk?);
            }
            self.inner
                .lock()
                .await
                .insert(key.to_string(), out.freeze());
            Ok(())
        }

        async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), std::io::Error> {
            let mut map = self.inner.lock().await;
            let value = map.remove(orig_key).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "source key not found")
            })?;
            map.insert(target_key.to_string(), value);
            Ok(())
        }
    }

    #[cfg(feature = "encode")]
    #[async_trait::async_trait]
    impl StorageReader for ChunkingMemStorage {
        async fn get(&self, key: &str, _: Option<Options>) -> Result<Object, std::io::Error> {
            let map = self.inner.lock().await;
            let bytes = map
                .get(key)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))?;
            let len = bytes.len() as u64;
            let step = self.chunk_len.max(1);
            let chunks = bytes
                .chunks(step)
                .map(|chunk| Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(chunk)))
                .collect::<Vec<_>>();
            let stream: ByteStream = Box::pin(stream::iter(chunks));
            Ok(Object {
                stream,
                content_length: Some(len),
                content_type: None,
                etag: None,
                last_modified: None,
            })
        }
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_aes_small_read_chunks() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(ChunkingMemStorage::new(3));
        let payload = b"encrypted-stream-partial-frames-should-not-stall".to_vec();
        let opts = Options::new([17u8; 32], [19u8; 12], 1, b"small-chunk-aad".to_vec());

        storage
            .write(
                "enc/chunked",
                Some(opts.clone()),
                stream_from_parts(vec![
                    payload[..11].to_vec(),
                    payload[11..23].to_vec(),
                    payload[23..].to_vec(),
                ]),
            )
            .await?;

        let obj = storage.get("enc/chunked", Some(opts)).await?;
        let got = tokio::time::timeout(Duration::from_secs(2), read_all(obj.stream))
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "decrypt stream stalled")
            })??;
        assert_eq!(got, payload);
        Ok(())
    }
}
