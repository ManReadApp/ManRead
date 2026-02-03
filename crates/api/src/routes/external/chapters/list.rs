use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission,
    resp::external::{ScrapeChapterListRequest, ScrapeChapterListResponse},
};
use apistos::api_operation;
use surrealdb_extras::{RecordIdType, SurrealTableInfo};

use crate::{
    error::ApiResult,
    models::{
        chapter::ChapterDBService, manga::Manga, scraper::ScraperDbService, version::Version,
    },
};

#[api_operation(
    tag = "external",
    summary = "Lists all the chapters that can be scraped for a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ScrapeChapterListRequest>,
    scraper_service: Data<ScraperDbService>,
    chapter_service: Data<ChapterDBService>,
) -> ApiResult<Json<Vec<ScrapeChapterListResponse>>> {
    let manga_id = RecordIdType::from((Manga::name(), data.manga_id.as_str()));
    let version_id = RecordIdType::from((Version::name(), data.version_id.as_str()));
    let mut to_scrape = scraper_service
        .get_items(manga_id.clone(), version_id.clone())
        .await?
        .into_iter()
        .map(|v| ScrapeChapterListResponse {
            id: v.id.to_string(),
            chapter: v.data.data.chapter,
            name: v.data.data.names,
            link: Some(v.data.data.url),
            state: v.data.state.to_string(),
        })
        .collect::<Vec<_>>();
    to_scrape.extend(
        chapter_service
            .get_chapter_by_version(manga_id, version_id)
            .await?
            .into_iter()
            .map(|(id, ch, name)| ScrapeChapterListResponse {
                id: id.to_string(),
                chapter: ch,
                name,
                link: None,
                state: "Processed".to_string(),
            }),
    );

    Ok(Json(to_scrape))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
