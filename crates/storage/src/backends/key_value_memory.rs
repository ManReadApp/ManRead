use std::{collections::HashMap, io};

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::RwLock;

use crate::backends::KeyValueStore;

pub struct InMemoryKeyValueStore<V> {
    inner: RwLock<HashMap<String, V>>,
}

impl<V> InMemoryKeyValueStore<V> {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl<V> Default for InMemoryKeyValueStore<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<V> KeyValueStore<V> for InMemoryKeyValueStore<V>
where
    V: Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    type Error = io::Error;

    async fn get(&self, key: &str) -> Result<Option<V>, Self::Error> {
        Ok(self.inner.read().await.get(key).cloned())
    }

    async fn set(&self, key: &str, value: V) -> Result<(), Self::Error> {
        self.inner.write().await.insert(key.to_owned(), value);
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<Option<V>, Self::Error> {
        Ok(self.inner.write().await.remove(key))
    }
}
