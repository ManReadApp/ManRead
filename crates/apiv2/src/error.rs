use std::{
    fmt::{Display, Formatter},
    io,
};

use actix_web::{http::StatusCode, rt::task::JoinError, ResponseError};
use apistos::ApiErrorComponent;
use db::error::DbError;
use serde::{Deserialize, Serialize};
use storage::StorageError;

#[derive(Serialize, Deserialize, Clone, Debug, ApiErrorComponent)]
#[openapi_error(status(code = 405, description = "Invalid input"))]
pub enum ApiError {
    PasswordIncorrect,
    NameExists,
    InvalidImageId,
    EmailExists,
    NoContentDisposition,
    ChapterVersionAlreadyExists,
    ExpiredToken,
    NotFoundInDB,
    WrongResetToken,
    WriteError(String),
    MultiPart(String),
    InvalidInput(String),
    InvalidActivationToken,
    Bcrypt(String),
    FailedToEncodeToken(String),
}

impl From<io::Error> for ApiError {
    fn from(value: io::Error) -> Self {
        ApiError::WriteError(value.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
impl From<DbError> for ApiError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::NotFound => ApiError::NotFoundInDB,
            DbError::InvalidActivationToken => ApiError::InvalidActivationToken,
            DbError::ExpiredToken => ApiError::ExpiredToken,
            DbError::NoImage => ApiError::invalid_input("No image available"),
            DbError::NoExtension => ApiError::invalid_input("could not extract file extension"),
            DbError::SearchParseError(error) => ApiError::invalid_input(&error),
            DbError::SurrealDbError(error) => ApiError::write_error(error.to_string()),
            DbError::DbError(storage_error) => storage_error.into(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        ApiError::FailedToEncodeToken(value.to_string())
    }
}

impl ApiError {
    pub fn db_error(msg: impl ToString) -> Self {
        Self::WriteError(msg.to_string())
    }

    pub fn generate_jwt(msg: jsonwebtoken::errors::Error) -> Self {
        ApiError::FailedToEncodeToken(msg.to_string())
    }

    pub fn write_error(msg: impl ToString) -> Self {
        Self::WriteError(msg.to_string())
    }
    pub fn invalid_input(msg: &str) -> Self {
        ApiError::InvalidInput(msg.to_string())
    }

    pub fn multipart_read_error(msg: impl ToString) -> Self {
        Self::MultiPart(msg.to_string())
    }
}

impl From<StorageError> for ApiError {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::HandleNotFound => ApiError::NotFoundInDB,
            StorageError::NoDefaultImageAvailable => {
                ApiError::invalid_input("No default image available")
            }
            StorageError::MissingExtension => ApiError::invalid_input("Missing file extension"),
            StorageError::Processing(processing_error) => {
                ApiError::write_error(processing_error.to_string())
            }
            StorageError::Io(error) => ApiError::write_error(error.to_string()),
            StorageError::TempFile(error) => ApiError::write_error(error.to_string()),
        }
    }
}

impl From<JoinError> for ApiError {
    fn from(value: JoinError) -> Self {
        ApiError::FailedToEncodeToken(value.to_string())
    }
}

impl From<bcrypt::BcryptError> for ApiError {
    fn from(value: bcrypt::BcryptError) -> Self {
        Self::Bcrypt(value.to_string())
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
