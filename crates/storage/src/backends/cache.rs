use std::{io, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::{
    backends::{ByteStream, StorageReader, StorageWriter},
    DiskStorage, Object, Options,
};

pub struct CacheBackend<S> {
    sr: DiskStorage,
    lock: KeyedLock<String>,
    inner: S,
}

impl<S> CacheBackend<S> {
    pub fn new(root: impl Into<PathBuf>, inner: S) -> Self {
        Self {
            lock: KeyedLock::new(),
            sr: DiskStorage::new(root.into().join("cache")),
            inner,
        }
    }
}

#[async_trait::async_trait]
impl<S: StorageWriter> StorageWriter for CacheBackend<S> {
    async fn write(&self, key: &str, stream: ByteStream) -> Result<(), io::Error> {
        self.inner.write(key, stream).await
    }

    async fn rename(&self, orig_key: &str, target_key: &str) -> Result<(), io::Error> {
        self.inner.rename(orig_key, target_key).await
    }

    async fn delete(&self, key: &str) -> Result<(), io::Error> {
        self.inner.delete(key).await?;
        match self.sr.delete(key).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
}

pub struct KeyedLock<K> {
    locks: DashMap<K, Arc<Mutex<()>>>,
}

impl<K> KeyedLock<K>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            locks: DashMap::new(),
        }
    }

    pub async fn lock(&self, key: K) -> OwnedMutexGuard<()> {
        let m = self
            .locks
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        m.lock_owned().await
    }
}

#[async_trait::async_trait]
impl<S: StorageReader> StorageReader for CacheBackend<S> {
    async fn get(&self, key: &str, options: &Options) -> Result<Object, std::io::Error> {
        match self.sr.get(key, options).await {
            Ok(obj) => return Ok(obj),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        let _guard = self.lock.lock(key.to_owned()).await;

        match self.sr.get(key, options).await {
            Ok(obj) => return Ok(obj),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        let obj = self.inner.get(key, options).await?;

        if options.cache_download {
            self.sr.write(key, obj.stream).await?;

            self.sr.get(key, options).await
        } else {
            Ok(obj)
        }
    }
}
