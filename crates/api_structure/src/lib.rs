pub mod auth;
pub mod error;
pub mod fonts;
pub mod home;
pub mod image;
pub mod info;
pub mod manga;
pub mod reader;
pub mod scrape;
pub mod scraper;
pub mod search;

use crate::error::{ApiErr, ApiErrorType};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::{ParseError, Url};

pub fn now_timestamp() -> Result<Duration, ApiErr> {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).map_err(|v| ApiErr {
        message: Some("Time went backwards".to_string()),
        cause: Some(v.to_string()),
        err_type: ApiErrorType::InternalError,
    })
}

pub trait RequestImpl {
    const ROUTE: &'static str;
    const AUTH: bool;
    const METHOD: &'static str = "POST";

    fn headers() -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("Content-Type".into(), "application/json".into());
        hm
    }

    fn request(url: &Url) -> Result<Request, ParseError> {
        Ok(Request {
            auth: Self::AUTH,
            url: url.join(Self::ROUTE)?,
            method: Self::METHOD.to_string(),
            headers: Self::headers(),
            req_body: vec![],
            bytes: false,
        })
    }
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

pub struct SearchUris;

impl RequestImpl for SearchUris {
    const ROUTE: &'static str = "external/search/sites";
    const AUTH: bool = true;
}
