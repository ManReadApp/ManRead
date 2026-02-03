use std::fmt::{Display, Formatter};

use actix_web::{error::BlockingError, http::StatusCode, ResponseError};
use apistos::ApiErrorComponent;
use serde::{Deserialize, Serialize};

use crate::models::logs::LogDbService;

pub type ApiResult<T> = std::result::Result<T, ApiError>;

pub struct ErrorLogger(ApiError);

impl Display for ErrorLogger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ApiError> for ErrorLogger {
    fn from(error: ApiError) -> Self {
        ErrorLogger(error)
    }
}

impl Drop for ErrorLogger {
    fn drop(&mut self) {
        let future = LogDbService::error(self.0.clone());
        tokio::spawn(future);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiErrorComponent)]
#[openapi_error(status(code = 405, description = "Invalid input"))]
pub enum ApiError {
    FailedToEncodeToken(String),
    SearchParseError(String),
    PasswordIncorrect,
    UriDoesNotExist,
    NotFoundInDB,
    FileNotFound,
    DoesNotSupportSearch,
    NoCoverTemplatesFound,
    NameExists,
    ChapterVersionAlreadyExists,
    WrongResetToken,
    EmailExists,
    InvalidActivationToken,
    TempFileNotFound,
    InvalidImageId,
    CannotSaveTempFile,
    InvalidAuthToken(String),
    BlockingError(String),
    NoFileExtensionForTempFile,
    MalformedTempFilename,
    NoContentDisposition,
    IoError(String),
    InvalidInput(String),
    BcryptError(String),
    ImageError(String),
    MultiPart(String),
    WriteError(String),
    ScraperError(String),
    SurrealDbError(String),
    ExpiredToken,
    External(String),
}

impl From<scraper_module::ScraperError> for ApiError {
    fn from(value: scraper_module::ScraperError) -> Self {
        Self::External(format!("{:?}", value))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::ScraperError(e.to_string())
    }
}

#[cfg(feature = "db")]
impl From<surrealdb::Error> for ApiError {
    fn from(e: surrealdb::Error) -> Self {
        ApiError::SurrealDbError(e.to_string())
    }
}

impl From<bcrypt::BcryptError> for ApiError {
    fn from(e: bcrypt::BcryptError) -> Self {
        ApiError::BcryptError(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        ApiError::InvalidAuthToken(e.to_string())
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

impl From<BlockingError> for ApiError {
    fn from(e: BlockingError) -> Self {
        ApiError::BlockingError(e.to_string())
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

impl From<image::ImageError> for ApiError {
    fn from(e: image::ImageError) -> Self {
        ApiError::ImageError(e.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::IoError(e.to_string())
    }
}
