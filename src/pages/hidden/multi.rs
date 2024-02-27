use std::collections::HashMap;
use api_structure::error::{ApiErr, ApiErrorType};
use crate::error::ScrapeError;
use crate::services::multisite::Info;

fn register() -> Vec<&'static str> {
    vec![]
}

pub fn post_process_pages(uri: &str, values: HashMap<String, String>) -> Result<Vec<String>, ScrapeError> {
    Err(ApiErr {
        message: Some("couldnt find fields to process".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }.into())
}

pub async fn manual_pages(info:Info) -> Result<Vec<String>, ScrapeError> {
    Err(ApiErr {
        message: Some("uri not registered".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }.into())
}

pub fn post_process_info(uri: &str, values: HashMap<String, String>) -> Result<Vec<Info>, ScrapeError> {
    Err(ApiErr {
        message: Some("couldnt find fields to process".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }.into())
}

pub async fn manual_info(uri: &str, url: &str) -> Result<Vec<Info>, ScrapeError> {
    Err(ApiErr {
        message: Some("uri not registered".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }.into())
}