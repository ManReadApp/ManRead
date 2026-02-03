use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, resp::external::ToScrape};
use apistos::api_operation;
use surrealdb_extras::RecordData;

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::{manga::MangaTitle, scraper::ScraperDbService},
};

#[api_operation(
    tag = "external",
    summary = "Gets the metadata of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    scraper_service: Data<ScraperDbService>,
) -> ApiResult<Json<Vec<ToScrape>>> {
    todo!()
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/info").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
