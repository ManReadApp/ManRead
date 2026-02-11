use crate::StorageError;

/// Simple key value store mapping internal ids to uuids
#[async_trait::async_trait]
pub trait KeyMapper<T: Send + Sync>: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Result<Option<T>, StorageError>;
    async fn set(&self, key: &str, value: T) -> Result<(), StorageError>;
    async fn remove(&self, key: &str) -> Result<Option<T>, StorageError>;
    async fn rename(&self, old_key: &str, new_key: &str) -> Result<(), StorageError> {
        let value = self.remove(old_key).await?;
        if let Some(v) = value {
            self.set(new_key, v).await?;
        }
        Ok(())
    }
}
