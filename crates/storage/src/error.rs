pub enum StorageError {
    HandleNotFound,
    NoDefaultImageAvailable,
    MissingExtension,
    OSError(std::io::Error),
}
pub type StorageResult<T> = Result<T, StorageError>;

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        Self::OSError(value)
    }
}

impl From<async_tempfile::Error> for StorageError {
    fn from(value: async_tempfile::Error) -> Self {
        todo!()
    }
}
