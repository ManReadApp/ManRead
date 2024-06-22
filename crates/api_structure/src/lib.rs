pub mod auth;
pub mod error;
pub mod models;
pub mod req;
pub mod resp;
pub mod scrape;
pub mod scraper;
pub mod search;

use crate::error::{ApiErr, ApiErrorType};
use req::fonts::{FontRequest, FontsRequest};
use req::manga::add::AddMangaRequest;
use req::manga::cover::MangaCoverRequest;
use req::manga::info::MangaInfoRequest;
use req::manga::search::SearchRequest;
use req::manga::tag::TagsRequest;
use req::manga::{HomeRequest, KindsRequest};
use req::reader::image::MangaReaderImageRequest;
use req::reader::info::MangaReaderRequest;
use req::reader::pages::ReaderPageRequest;
use resp::manga::home::HomeResponse;
use resp::manga::info::MangaInfoResponse;
use resp::manga::search::SearchResponse;
use resp::manga::{KindsResponse, TagsResponse};
use resp::reader::pages::ReaderPageResponse;
use resp::reader::MangaReaderResponse;
use resp::{ByteResponse, FontsResponse, NoResponse};
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

macro_rules! request {
    ($name:ident, $route:expr, $auth:expr, $out:ty) => {
        impl crate::RequestImpl for $name {
            const ROUTE: &'static str = proc_macros::strip_prefix!($route);
            const AUTH: bool = $auth;
        }

        impl $name {
            fn respone_type() -> std::marker::PhantomData<$out> {
                std::marker::PhantomData
            }
        }
    };
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
            url: url.join(&Self::ROUTE[1..])?,
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

request!(FontRequest, "/font", true, NoResponse);
request!(FontsRequest, "/fonts", true, FontsResponse);

request!(HomeRequest, "/home", true, HomeResponse);
request!(AddMangaRequest, "/manga/add", true, NoResponse);
request!(KindsRequest, "/manga/kinds", true, KindsResponse);
request!(TagsRequest, "/manga/tags", true, TagsResponse);
request!(MangaInfoRequest, "/manga/info", true, MangaInfoResponse);
request!(SearchRequest, "/manga/search", true, SearchResponse);
request!(MangaCoverRequest, "/manga/cover", true, ByteResponse);

request!(
    MangaReaderImageRequest,
    "/reader/chapter_page",
    true,
    ByteResponse
);
request!(
    MangaReaderRequest,
    "/reader/info",
    true,
    MangaReaderResponse
);
request!(ReaderPageRequest, "/reader/pages", true, ReaderPageResponse);
