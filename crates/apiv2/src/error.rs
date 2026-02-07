use std::fmt::{Display, Formatter};

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
    EmailExists,
    NoContentDisposition,
    ExpiredToken,
    DbError(DbError),
    WrongResetToken,
    WriteError(String),
    MultiPart(String),
    InvalidInput(String),
    InvalidActivationToken,
    Bcrypt(String),
    FailedToEncodeToken(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        ApiError::FailedToEncodeToken(value.to_string())
    }
}

impl ApiError {
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
            StorageError::HandleNotFound => todo!(),
            StorageError::NoDefaultImageAvailable => todo!(),
            StorageError::MissingExtension => todo!(),
            StorageError::OSError(error) => todo!(),
        }
    }
}

impl From<JoinError> for ApiError {
    fn from(value: JoinError) -> Self {
        ApiError::FailedToEncodeToken(value.to_string())
    }
}

impl From<DbError> for ApiError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::NotFound => todo!(),
            DbError::InvalidActivationToken => todo!(),
            DbError::ExpiredToken => todo!(),
            DbError::NoImage => todo!(),
            DbError::NoExtension => todo!(),
            DbError::SearchParseError(_) => todo!(),
            DbError::SurrealDbError(error) => todo!(),
            DbError::DbError(storage_error) => todo!(),
        }
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
