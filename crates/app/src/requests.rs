use api_structure::req::auth::activate::ActivateRequest;
use api_structure::req::auth::login::LoginRequest;
use api_structure::req::auth::register::RegisterRequest;
use api_structure::req::auth::reset_password::{RequestResetPasswordRequest, ResetPasswordRequest};
use api_structure::req::auth::TokenRefreshRequest;
use api_structure::resp::auth::JWTsResponse;
use paste::paste;
use std::collections::HashMap;
use std::sync::Arc;

use api_structure::req::fonts::{FontRequest, FontsRequest};
use api_structure::req::manga::add::AddMangaRequest;
use api_structure::req::manga::cover::MangaCoverRequest;
use api_structure::req::manga::external_search::ExternalSearchRequest;
use api_structure::req::manga::info::MangaInfoRequest;
use api_structure::req::manga::search::SearchRequest;
use api_structure::req::manga::tag::TagsRequest;
use api_structure::req::manga::{AvailableExternalSitesRequest, HomeRequest, KindsRequest};
use api_structure::req::reader::image::MangaReaderImageRequest;
use api_structure::req::reader::info::MangaReaderRequest;
use api_structure::req::reader::pages::ReaderPageRequest;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use api_structure::resp::manga::home::HomeResponse;
use api_structure::resp::manga::info::MangaInfoResponse;
use api_structure::resp::manga::search::SearchResponse;
use api_structure::resp::manga::{KindsResponse, TagsResponse};
use api_structure::resp::reader::pages::ReaderPageResponse;
use api_structure::resp::reader::MangaReaderResponse;
use api_structure::resp::{
    AvailableExternalSitesResponse, ByteResponse, FontsResponse, NoResponse,
};
use api_structure::Request;
use serde::de::DeserializeOwned;
use url::ParseError;

use crate::fetcher::Fetcher;

pub trait RequestImpl<T: Send + DeserializeOwned> {
    const ROUTE: &'static str;
    const AUTH: bool;
    const METHOD: &'static str = "POST";

    fn fetcher(url: &url::Url) -> Fetcher<T> {
        Fetcher::new(Self::request(url).unwrap())
    }

    fn fetcher_ctx(url: &url::Url, ctx: egui::Context) -> Fetcher<T> {
        Fetcher::new_ctx(Self::request(url).unwrap(), ctx)
    }

    fn cfetcher_ctx<V: Send + DeserializeOwned>(url: &url::Url, ctx: egui::Context) -> Fetcher<V> {
        Fetcher::new_ctx(Self::request(url).unwrap(), ctx)
    }

    fn headers() -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("Content-Type".into(), "application/json".into());
        hm
    }

    fn request(url: &url::Url) -> Result<Request, ParseError> {
        Ok(Request {
            auth: Self::AUTH,
            url: url.join(&Self::ROUTE)?,
            method: Self::METHOD.to_string(),
            headers: Self::headers(),
            req_body: vec![],
            bytes: false,
        })
    }
}

macro_rules! request {
    ($name:ident, $route:expr, $auth:expr, $out:ty) => {
        impl RequestImpl<$out> for $name {
            const ROUTE: &'static str = proc_macros::strip_prefix!($route);
            const AUTH: bool = $auth;
        }
        paste! {
            #[allow(unused)]
            pub type [<$name Fetcher>] = Fetcher<$out>;
        }
    };
}
// Fonts
request!(FontRequest, "/fonts/file", false, NoResponse);
request!(FontsRequest, "/fonts/list", false, FontsResponse);

request!(ActivateRequest, "/auth/activate", true, JWTsResponse);
request!(TokenRefreshRequest, "/refresh", true, JWTsResponse);
request!(LoginRequest, "/auth/sign_in", false, JWTsResponse);
request!(RegisterRequest, "/auth/sign_up", false, JWTsResponse);
request!(
    ResetPasswordRequest,
    "/auth/reset_password",
    false,
    JWTsResponse
);
request!(
    RequestResetPasswordRequest,
    "/auth/request_reset_password",
    false,
    NoResponse
);

//todo: upload
//todo: spinner

request!(HomeRequest, "/home", true, Arc<HomeResponse>);

request!(AddMangaRequest, "/manga/add", true, String); //TODO: implement
request!(KindsRequest, "/manga/kinds", true, KindsResponse);
request!(TagsRequest, "/manga/tags", true, TagsResponse);
request!(MangaInfoRequest, "/manga/info", true, MangaInfoResponse);
request!(SearchRequest, "/manga/search", true, Vec<SearchResponse>);
request!(MangaCoverRequest, "/manga/cover", true, ByteResponse);
request!(
    ExternalSearchRequest,
    "/manga/search/external",
    true,
    Vec<ScrapeSearchResponse>
);
request!(
    AvailableExternalSitesRequest,
    "/manga/search/external/list",
    true,
    AvailableExternalSitesResponse
);

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
    Arc<MangaReaderResponse>
);
request!(
    ReaderPageRequest,
    "/reader/pages",
    true,
    Arc<ReaderPageResponse>
);
// request!(
//     MangaReaderTranslationRequest,
//     "/reader/translation",
//     true,
//     Vec<TranslationArea>
// );
