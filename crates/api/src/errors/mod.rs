mod bcript;
mod defaults;
mod image;
mod io;
mod json;
mod scrape;
mod surreal;

use std::fmt::{Display, Formatter};

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use api_structure::error::{ApiErr, ApiErrorType};

pub(crate) struct ApiError(ApiErr);

pub type ApiResult<T> = Result<T, ApiError>;
mod debugging {
    use crate::errors::ApiError;
    use std::fmt::{Debug, Formatter};

    impl Debug for ApiError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.0.err_type {
            ApiErrorType::NotFoundError => StatusCode::NOT_FOUND,
            ApiErrorType::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ReadError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorType::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiErrorType::WriteError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorInvalidUrl => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorJsSandboxError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorBase64Error => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorKeyDecryptionError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorInputError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorFetchError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorParseError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorReadError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorCurl => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::ScrapeErrorStatus => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(&self.0)
    }
}

impl From<ApiErr> for ApiError {
    fn from(value: ApiErr) -> Self {
        ApiError(value)
    }
}
