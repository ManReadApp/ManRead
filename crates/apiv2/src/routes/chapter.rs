use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{ChapterInfoResponse, IdRequest, NewChapterRequest},
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};
use chrono::DateTime;

use crate::{actions::chapter::ChapterActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapter")
        .service(
            apistos::web::resource("/add").route(
                apistos::web::put()
                    .to(add)
                    .guard(AuthorityGuard::new(Permission::Create)),
            ),
        )
        .service(
            apistos::web::resource("/delete").route(
                apistos::web::delete()
                    .to(delete)
                    .guard(AuthorityGuard::new(Permission::RequestDelete)),
            ),
        )
        .service(
            apistos::web::resource("/edit").route(
                apistos::web::put()
                    .to(edit)
                    .guard(AuthorityGuard::new(Permission::Create)),
            ),
        )
        .service(
            apistos::web::resource("/info").route(
                apistos::web::post()
                    .to(info)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
}
#[api_operation(
    tag = "chapter",
    summary = "Returns info about a chapter",
    description = r###""###
)]
pub(crate) async fn info(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<Json<ChapterInfoResponse>> {
    chapter_service.info(&data.id).await.map(Json)
}

#[api_operation(
    tag = "chapter",
    summary = "Adds a chapter to an existing chapter",
    description = r###""###
)]
pub(crate) async fn add(
    Json(chapter): Json<NewChapterRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<CreatedJson<u8>> {
    chapter_service
        .add(
            &chapter.manga_id,
            chapter.titles,
            chapter.episode,
            &chapter.version,
            chapter.images,
            chapter.tags,
            chapter.sources,
            chapter
                .release_date
                .map(|v| DateTime::from_timestamp_millis(v as i64).unwrap()),
        )
        .await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "chapter",
    summary = "Deletes a chapter",
    description = r###""###
)]
pub(crate) async fn delete(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<CreatedJson<u8>> {
    chapter_service.delete(&data.id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "chapter",
    summary = "Modifies a chapter",
    description = r###""###
)]
pub(crate) async fn edit() -> CreatedJson<String> {
    //TODO: impl
    CreatedJson("Hello World".to_owned())
}
