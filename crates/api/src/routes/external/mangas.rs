use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission, req::PaginationRequest, resp::external::ToScrape,
};
use apistos::api_operation;
use surrealdb_extras::RecordData;

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::{manga::MangaTitle, scraper::ScraperDbService},
};

#[api_operation(
    tag = "external",
    summary = "Lists all the mangas that need to be processed(chapters)",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(_): Json<PaginationRequest>,
    scraper_service: Data<ScraperDbService>,
) -> ApiResult<Json<Vec<ToScrape>>> {
    let mut data = vec![];
    let items = scraper_service.process_items().await?;
    for (m, v) in items {
        let manga: RecordData<MangaTitle> =
            m.get_part(&*DB).await?.ok_or(ApiError::NotFoundInDB)?;
        let v = v.get(&*DB).await?.ok_or(ApiError::NotFoundInDB)?;
        data.push(ToScrape {
            manga_id: manga.id.id().to_string(),
            names: manga.data.titles,
            version: v.data.name,
            version_id: v.id.id().to_string(),
        })
    }
    Ok(Json(data))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/mangas").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
