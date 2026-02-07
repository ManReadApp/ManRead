#[cfg(feature = "encode")]
mod aes_gcm;
#[cfg(feature = "disk")]
mod disk;
mod memory;

#[cfg(feature = "encode")]
pub use aes_gcm::EncryptedStorage;
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
