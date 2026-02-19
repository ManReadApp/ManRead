#[async_trait::async_trait]
pub trait KeyValueStore<V: Send + Sync + 'static>: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, key: &str) -> Result<Option<V>, Self::Error>;
    async fn set(&self, key: &str, value: V) -> Result<(), Self::Error>;
    async fn remove(&self, key: &str) -> Result<Option<V>, Self::Error>;

    async fn rename(&self, old_key: &str, new_key: &str) -> Result<(), Self::Error> {
        let value = self.remove(old_key).await?;
        if let Some(v) = value {
            self.set(new_key, v).await?;
        }
        Ok(())
    }
}
