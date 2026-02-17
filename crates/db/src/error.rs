use std::fmt::Display;

use storage::StorageError;

#[derive(Debug)]
pub enum DbError {
    NotFound,
    InvalidActivationToken,
    ExpiredToken,
    NoImage,
    NoExtension,
    SearchParseError(String),
    SurrealDbError(surrealdb::Error),
    DbError(StorageError),
}
pub type DbResult<T> = Result<T, DbError>;

impl From<StorageError> for DbError {
    fn from(value: StorageError) -> Self {
        Self::DbError(value)
    }
}

impl From<surrealdb::Error> for DbError {
    fn from(value: surrealdb::Error) -> Self {
        DbError::SurrealDbError(value)
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "Not Found in database"),
            DbError::SearchParseError(error) => {
                write!(f, "failed to parse search query: {}", error)
            }
            DbError::SurrealDbError(error) => write!(f, "SurrealDB Error: {}", error),
            DbError::InvalidActivationToken => write!(f, "Invalid activation token"),
            DbError::ExpiredToken => write!(f, "Token expired"),
            DbError::NoImage => write!(f, "No image metadata available"),
            DbError::NoExtension => write!(f, "No file extension available"),
            DbError::DbError(storage_error) => write!(f, "Storage error: {}", storage_error),
        }
    }
}
