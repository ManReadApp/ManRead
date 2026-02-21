use serde::{de::DeserializeOwned, Serialize};
use storage::KeyValueStore;
use surrealdb_extras::RecordIdFunc;

use crate::{
    error::{DbError, DbResult},
    tag::Empty,
    DbSession,
};

pub struct KeyValueDb {
    name: String,
    db: DbSession,
}

#[async_trait::async_trait]
impl<V> KeyValueStore<V> for KeyValueDb
where
    V: Send + Sync + 'static + Serialize + DeserializeOwned,
{
    type Error = DbError;

    async fn get(&self, key: &str) -> Result<Option<V>, Self::Error> {
        self.get(key).await
    }
    async fn set(&self, key: &str, value: V) -> Result<(), Self::Error> {
        self.set(key, value).await
    }
    async fn remove(&self, key: &str) -> Result<Option<V>, Self::Error> {
        self.remove(key).await
    }
}

impl KeyValueDb {
    pub fn new(name: &str, db: DbSession) -> Self {
        KeyValueDb {
            name: format!("kv_{}", name),
            db,
        }
    }

    pub async fn set<T: Serialize + 'static>(&self, key: &str, value: T) -> DbResult<()> {
        let _: Option<Empty> = self
            .db
            .upsert((self.name.as_str(), key))
            .content(value)
            .await?;
        Ok(())
    }

    pub async fn remove<T: DeserializeOwned>(&self, key: &str) -> DbResult<Option<T>> {
        Ok(RecordIdFunc::from((self.name.as_str(), key))
            .delete(self.db.as_ref())
            .await?)
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> DbResult<Option<T>> {
        Ok(RecordIdFunc::from((self.name.as_str(), key))
            .get(self.db.as_ref())
            .await?)
    }
}
