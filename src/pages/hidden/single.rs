use crate::error::ScrapeError;
use api_structure::error::{ApiErr, ApiErrorType};
use std::collections::HashMap;

pub fn register() -> Vec<&'static str> {
    vec![]
}

pub fn post_process(
    _uri: &str,
    _values: HashMap<String, String>,
) -> Result<Vec<String>, ScrapeError> {
    Err(ApiErr {
        message: Some("couldnt find fields to process".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }
    .into())
}

pub async fn manual(_uri: &str, _url: &str) -> Result<Vec<String>, ScrapeError> {
    Err(ApiErr {
        message: Some("uri not registered".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }
    .into())
}
