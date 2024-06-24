use crate::errors::ApiResult;
use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web_grants::protect;
use api_structure::models::manga::external_search::ValidSearches;
use api_structure::req::manga::external_search::ExternalSearchRequest;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use log::debug;
use manread_scraper::SearchService;
use std::collections::HashMap;

#[post("/search/external/list")]
#[protect(
    any(
        "api_structure::models::auth::role::Role::Admin",
        "api_structure::models::auth::role::Role::CoAdmin",
        "api_structure::models::auth::role::Role::Moderator",
        "api_structure::models::auth::role::Role::Author",
        "api_structure::models::auth::role::Role::User"
    ),
    ty = "api_structure::models::auth::role::Role"
)]
pub async fn available_external_search_sites(
    search_service: Data<SearchService>,
) -> Json<HashMap<String, ValidSearches>> {
    Json(search_service.sites())
}

#[post("/search/external")]
#[protect(
    any(
        "api_structure::models::auth::role::Role::Admin",
        "api_structure::models::auth::role::Role::CoAdmin",
        "api_structure::models::auth::role::Role::Moderator",
        "api_structure::models::auth::role::Role::Author",
        "api_structure::models::auth::role::Role::User"
    ),
    ty = "api_structure::models::auth::role::Role"
)]
pub async fn search(
    Json(data): Json<ExternalSearchRequest>,
    search_service: Data<SearchService>,
) -> ApiResult<Json<Vec<ScrapeSearchResponse>>> {
    debug!("External Search Uri: {:?}", data.uri);
    Ok(Json(search_service.search(&data.uri, data.data).await?))
}
