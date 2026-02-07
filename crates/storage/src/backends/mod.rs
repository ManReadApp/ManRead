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
    use bytes::Bytes;
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
}
