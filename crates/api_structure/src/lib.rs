pub mod error;
pub mod models;
pub mod req;
pub mod resp;
pub mod scrape;
pub mod search;

use crate::error::{ApiErr, ApiErrorType};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

pub fn now_timestamp() -> Result<Duration, ApiErr> {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).map_err(|v| ApiErr {
        message: Some("Time went backwards".to_string()),
        cause: Some(v.to_string()),
        err_type: ApiErrorType::InternalError,
    })
}

pub struct Request {
    pub auth: bool,
    pub url: Url,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub req_body: Vec<u8>,
    pub bytes: bool,
}

impl Request {
    pub fn set_content(&mut self, s: String) {
        self.req_body = s.as_bytes().to_vec();
    }
}

use crate::req::reader::image::MangaReaderImageRequest;
pub struct MangaReaderTranslationRequest(pub MangaReaderImageRequest);
