use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{api_operation, ApiComponent};
use manga_scraper::init::Services;
use schemars::JsonSchema;
use scraper_module::{ExternalSearchResponse, ScrapedSearchResponse, SearchQuery};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};

#[derive(ApiComponent, JsonSchema, Deserialize)]
pub struct ExternalSearch {
    query: SearchQuery,
    uri: String,
}
#[api_operation(
    tag = "external",
    summary = "Searches for mangas",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ExternalSearch>,
    services: Data<Services>,
) -> ApiResult<Json<ExternalSearchResponse>> {
    let service = services
        .get_by_uri(&data.uri)
        .ok_or(ApiError::UriDoesNotExist)?;
    let search = service
        .searchers
        .as_ref()
        .ok_or(ApiError::DoesNotSupportSearch)?;
    let search = search.search(data.query).await?;
    Ok(Json(search))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/search").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
