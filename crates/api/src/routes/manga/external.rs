//

use crate::errors::ApiResult;
use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web_grants::protect;
use api_structure::scraper::{ExternalSearchRequest, ScrapeSearchResult, ValidSearches};
use log::debug;
use manread_scraper::SearchService;
use std::collections::HashMap;

#[post("/external/search/sites")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn available_external_search_sites(
    search_service: Data<SearchService>,
) -> Json<HashMap<String, ValidSearches>> {
    Json(search_service.sites())
}

#[post("/search/external")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn search(
    Json(data): Json<ExternalSearchRequest>,
    search_service: Data<SearchService>,
) -> ApiResult<Json<Vec<ScrapeSearchResult>>> {
    debug!("External Search Uri: {:?}", data.uri);
    Ok(Json(search_service.search(&data.uri, data.data).await?))
}
