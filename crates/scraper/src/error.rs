use api_structure::error::{ApiErr, ApiErrorType};
use base64::DecodeError;
use js_sandbox::JsError;
use openssl::error::ErrorStack;
use std::io;
use std::io::Error;
use std::num::ParseFloatError;
use std::str::Utf8Error;

#[derive(Debug)]
pub struct ScrapeError(pub ApiErr);

impl From<ApiErr> for ScrapeError {
    fn from(value: ApiErr) -> Self {
        Self(value)
    }
}

#[cfg(feature = "curl")]
pub fn status_code_err(code: u32) -> ScrapeError{
    ScrapeError(ApiErr {
        message: Some(format!("failed with status code {code}")),
        cause: None,
        err_type: ApiErrorType::ScrapeErrorStatus,
    })
}

impl From<curl::Error> for ScrapeError {
    fn from(value: curl::Error) -> Self {
        ScrapeError(ApiErr {
            message: Some("failed the curl request".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorCurl,
        })
    }
}

impl From<JsError> for ScrapeError {
    fn from(value: JsError) -> Self {
        ScrapeError(ApiErr {
            message: Some("failed to execute js".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorJsSandboxError,
        })
    }
}

impl From<DecodeError> for ScrapeError {
    fn from(value: DecodeError) -> Self {
        ScrapeError(ApiErr {
            message: Some("failed to decode base64".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorBase64Error,
        })
    }
}

impl From<ErrorStack> for ScrapeError {
    fn from(value: ErrorStack) -> Self {
        ScrapeError(ApiErr {
            message: Some("failed to decrypt key".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorKeyDecryptionError,
        })
    }
}

impl ScrapeError {
    pub fn input_error(msg: impl ToString) -> Self {
        ScrapeError(ApiErr {
            message: Some(msg.to_string()),
            cause: None,
            err_type: ApiErrorType::ScrapeErrorInputError,
        })
    }
    pub fn node_not_found() -> Self {
        ScrapeError(ApiErr {
            message: Some("didnt find node".to_string()),
            cause: None,
            err_type: ApiErrorType::ScrapeErrorInputError,
        })
    }

    pub fn invalid_url(msg: impl ToString) -> Self {
        ScrapeError(ApiErr {
            message: Some(msg.to_string()),
            cause: None,
            err_type: ApiErrorType::ScrapeErrorInputError,
        })
    }
}

impl From<io::Error> for ScrapeError {
    fn from(value: Error) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to read file".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorFetchError,
        })
    }
}

impl From<reqwest::Error> for ScrapeError {
    fn from(value: reqwest::Error) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to fetch data".to_string()),
            cause: Some(value.to_string()),
            err_type: ApiErrorType::ScrapeErrorFetchError,
        })
    }
}

impl From<ParseFloatError> for ScrapeError {
    fn from(error: ParseFloatError) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to parse float".to_string()),
            cause: Some(error.to_string()),
            err_type: ApiErrorType::ScrapeErrorParseError,
        })
    }
}

impl From<Utf8Error> for ScrapeError {
    fn from(error: Utf8Error) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to parse utf8".to_string()),
            cause: Some(error.to_string()),
            err_type: ApiErrorType::ScrapeErrorParseError,
        })
    }
}

impl From<serde_json::Error> for ScrapeError {
    fn from(error: serde_json::Error) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to parse json".to_string()),
            cause: Some(error.to_string()),
            err_type: ApiErrorType::ScrapeErrorParseError,
        })
    }
}

impl From<std::num::ParseIntError> for ScrapeError {
    fn from(error: std::num::ParseIntError) -> Self {
        ScrapeError(ApiErr {
            message: Some("Failed to parse int".to_string()),
            cause: Some(error.to_string()),
            err_type: ApiErrorType::ScrapeErrorParseError,
        })
    }
}
