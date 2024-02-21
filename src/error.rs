use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiErr {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub err_type: ApiErrorType,
}

impl ApiErr {
    pub fn message(&self) -> String {
        if matches!(self.err_type, ApiErrorType::NotFoundError) {
            return "Page not found".to_string();
        }
        if let Some(message) = &self.message {
            return message.to_string();
        }
        if let Some(err) = &self.cause {
            return err.to_string();
        }

        "An unexpected error occurred".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientError {
    pub message: String,
    pub cause: Option<String>,
    pub data: Option<String>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum ApiErrorType {
    InternalError,
    NotFoundError,
    InvalidInput,
    Unauthorized,
    ReadError,
    WriteError,
    ScrapeErrorInvalidUrl,
    ScrapeErrorJsSandboxError,
    ScrapeErrorBase64Error,
    ScrapeErrorKeyDecryptionError,
    ScrapeErrorInputError,
    ScrapeErrorFetchError,
    ScrapeErrorParseError,
    ScrapeErrorReadError
}

impl Display for ApiErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApiErrorType::NotFoundError => write!(f, "NotFoundError"),
            ApiErrorType::InternalError => write!(f, "InternalError"),
            ApiErrorType::InvalidInput => write!(f, "InvalidInput"),
            ApiErrorType::Unauthorized => write!(f, "Unauthorized"),
            ApiErrorType::ReadError => write!(f, "ReadError"),
            ApiErrorType::WriteError => write!(f, "WriteError"),
            ApiErrorType::ScrapeErrorInvalidUrl => write!(f, "ScrapeErrorInvalidUrl"),
            ApiErrorType::ScrapeErrorJsSandboxError => write!(f, "ScrapeErrorJsSandboxError"),
            ApiErrorType::ScrapeErrorBase64Error => write!(f, "ScrapeErrorBase64Error"),
            ApiErrorType::ScrapeErrorKeyDecryptionError => write!(f, "ScrapeErrorKeyDecryptionError"),
            ApiErrorType::ScrapeErrorInputError => write!(f, "ScrapeErrorInputError"),
            ApiErrorType::ScrapeErrorFetchError => write!(f, "ScrapeErrorFetchError"),
            ApiErrorType::ScrapeErrorParseError => write!(f, "ScrapeErrorParseError"),
            ApiErrorType::ScrapeErrorReadError => write!(f, "ScrapeErrorReadError"),
        }
    }
}

impl Display for ApiErr {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}
