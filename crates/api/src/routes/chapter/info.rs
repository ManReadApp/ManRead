use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission, req::IdRequest, resp::chapter::info::ChapterInfoResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{chapter::ChapterDBService, tag::TagDBService},
};

#[api_operation(
    tag = "chapter",
    summary = "Returns info about a chapter",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterDBService>,
    tag_service: Data<TagDBService>,
) -> ApiResult<CreatedJson<ChapterInfoResponse>> {
    let chapter = chapter_service.get_by_id(&data.id).await?;
    let tags = tag_service
        .get_tags(chapter.tags.into_iter().map(|v| v.thing.id().to_string()))
        .await?;
    Ok(CreatedJson(ChapterInfoResponse {
        titles: chapter.titles,
        chapter: chapter.chapter,
        tags,
        sources: chapter.sources,
        release_date: chapter.release_date.map(|v| v.to_string()),
        versions: chapter
            .versions
            .into_iter()
            .map(|v| (v.0, v.1.thing.id().to_string()))
            .collect(),
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/info").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
