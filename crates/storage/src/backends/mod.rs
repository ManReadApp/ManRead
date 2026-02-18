#[cfg(feature = "encode")]
mod aes_gcm;
mod cache;
mod content_length;
mod delay;
#[cfg(feature = "disk")]
mod disk;
mod key_value;
mod memory;
#[cfg(feature = "s3")]
mod s3;

#[cfg(feature = "encode")]
pub use aes_gcm::EncryptedStorage;
pub use cache::CacheBackend;
pub use content_length::ContentLengthStorage;
pub use delay::DelayStorage;
#[cfg(feature = "disk")]
pub use disk::DiskStorage;
pub use key_value::KeyValueStore;
pub use memory::MemStorage;
use rand::{rngs::OsRng, TryRngCore};
#[cfg(feature = "s3")]
pub use s3::{S3Storage, S3StorageOptions, S3UploadAcl};

use std::{pin::Pin, time::SystemTime};

use bytes::Bytes;
use futures_core::Stream;

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub struct Object {
    pub stream: ByteStream,
    pub content_length: Option<u64>,
    pub content_type: Option<mime::Mime>,
    pub etag: Option<String>,
    pub last_modified: Option<SystemTime>,
}

//TODO: cache policy: when set cache_download => on manga image + download next 2 chapters and 2 prev; cleanup cache: on next chapter

#[derive(Clone, Debug)]
pub struct AesOptions {
    key: [u8; 32],
    nonce: [u8; 12],
    counter: u32,
    aad: Vec<u8>,
}

impl AesOptions {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];

        OsRng
            .try_fill_bytes(&mut key)
            .map_err(|err| std::io::Error::other(format!("aes key generation failed: {err}")))?;
        OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|err| std::io::Error::other(format!("aes nonce generation failed: {err}")))?;

        Ok(Self {
            key,
            nonce,
            aad: Vec::new(),
            counter: 0,
        })
    }
}

#[derive(Clone)]
pub struct Options {
    pub cache_download: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            cache_download: false,
        }
    }
}

#[async_trait::async_trait]
pub trait StorageReader: Send + Sync + 'static {
    /// reads file as stream
    async fn get(&self, key: &str, options: &Options) -> Result<Object, std::io::Error>;
}

#[async_trait::async_trait]
pub trait StorageWriter: Send + Sync + 'static {
    /// add new object
    async fn write(&self, key: &str, stream: ByteStream) -> Result<(), std::io::Error>;

    /// already uploaded under a different key(old key gets removed and added under new key)
    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), std::io::Error>;

    /// removes an object by key
    async fn delete(&self, key: &str) -> Result<(), std::io::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};
    use futures_util::{stream, StreamExt as _};
    use std::time::{Duration, Instant};

    #[cfg(feature = "encode")]
    use crate::{backends::KeyValueStore, StorageError};

    #[cfg(feature = "encode")]
    #[derive(Default)]
    struct TestAesMapper {
        inner: tokio::sync::Mutex<std::collections::HashMap<String, AesOptions>>,
    }

    #[cfg(feature = "encode")]
    #[async_trait::async_trait]
    impl KeyValueStore<AesOptions> for TestAesMapper {
        type Error = StorageError;

        async fn get(&self, key: &str) -> Result<Option<AesOptions>, StorageError> {
            Ok(self.inner.lock().await.get(key).cloned())
        }

        async fn set(&self, key: &str, value: AesOptions) -> Result<(), StorageError> {
            self.inner.lock().await.insert(key.to_owned(), value);
            Ok(())
        }

        async fn remove(&self, key: &str) -> Result<Option<AesOptions>, StorageError> {
            Ok(self.inner.lock().await.remove(key))
        }
    }

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
        options: &Options,
    ) -> Result<(), std::io::Error>
    where
        T: StorageReader + StorageWriter,
    {
        storage
            .write(
                key,
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
        assert_roundtrip(&storage, "plain/mem", payload, &Options::default()).await
    }

    #[tokio::test]
    async fn roundtrip_mem_with_delay_no_aes() -> Result<(), std::io::Error> {
        let storage = DelayStorage::new(MemStorage::new(), Duration::from_millis(20));
        let payload = b"delayed-memory-roundtrip-with-multiple-chunks";

        let started = Instant::now();
        assert_roundtrip(&storage, "plain/delayed", payload, &Options::default()).await?;
        assert!(
            started.elapsed() >= Duration::from_millis(35),
            "delay wrapper should apply delay to write and read paths"
        );
        Ok(())
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_aes() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(MemStorage::new(), TestAesMapper::default());
        let payload = b"encrypted-memory-roundtrip-with-multiple-chunks";

        assert_roundtrip(&storage, "enc/mem", payload, &Options::default()).await
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_delay_and_aes_and_rename() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(
            DelayStorage::new(MemStorage::new(), Duration::from_millis(10)),
            TestAesMapper::default(),
        );
        let payload = b"encrypted-delayed-memory-roundtrip-with-multiple-chunks";

        storage
            .write(
                "tmp/key",
                stream_from_parts(vec![
                    payload[..13].to_vec(),
                    payload[13..29].to_vec(),
                    payload[29..].to_vec(),
                ]),
            )
            .await?;
        storage.rename("tmp/key", "final/key").await?;

        let obj = storage.get("final/key", &Options::default()).await?;
        let got = read_all(obj.stream).await?;
        assert_eq!(got.as_slice(), payload);
        Ok(())
    }

    #[cfg(feature = "encode")]
    #[tokio::test]
    async fn roundtrip_mem_with_aes_internal_options() -> Result<(), std::io::Error> {
        let storage = EncryptedStorage::new(MemStorage::new(), TestAesMapper::default());
        let payload = b"encrypted-memory-roundtrip-with-internal-options";

        assert_roundtrip(
            &storage,
            "enc/mem/internal-options",
            payload,
            &Options::default(),
        )
        .await
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
        async fn write(&self, key: &str, mut stream: ByteStream) -> Result<(), std::io::Error> {
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

        async fn delete(&self, key: &str) -> Result<(), std::io::Error> {
            let mut map = self.inner.lock().await;
            map.remove(key).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "source key not found")
            })?;
            Ok(())
        }
    }

    #[cfg(feature = "encode")]
    #[async_trait::async_trait]
    impl StorageReader for ChunkingMemStorage {
        async fn get(&self, key: &str, _: &Options) -> Result<Object, std::io::Error> {
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
        let storage = EncryptedStorage::new(ChunkingMemStorage::new(3), TestAesMapper::default());
        let payload = b"encrypted-stream-partial-frames-should-not-stall".to_vec();

        storage
            .write(
                "enc/chunked",
                stream_from_parts(vec![
                    payload[..11].to_vec(),
                    payload[11..23].to_vec(),
                    payload[23..].to_vec(),
                ]),
            )
            .await?;

        let obj = storage.get("enc/chunked", &Options::default()).await?;
        let got = tokio::time::timeout(Duration::from_secs(2), read_all(obj.stream))
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "decrypt stream stalled")
            })??;
        assert_eq!(got, payload);
        Ok(())
    }
}
