use crate::StorageError;

/// Simple key value store mapping internal ids to uuids
pub trait KeyMapper {
    async fn get(&self, key: &str) -> Result<String, StorageError>;
    async fn set(&self, key: &str) -> Result<String, StorageError>;
    async fn remove(&self, key: &str) -> Result<String, StorageError>;
}
