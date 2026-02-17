use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{
        ChapterVersionDeleteRequest, ChapterVersionEditRequest, PaginationRequest,
        VersionInfoResponse,
    },
    Permission,
};
use apistos::api_operation;

use crate::{actions::chapter_version::ChapterVersionActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapter-versions")
        .service(
            apistos::web::resource("/list").route(
                apistos::web::post()
                    .to(list)
                    .guard(AuthorityGuard::new(Permission::Read)),
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
}

#[api_operation(
    tag = "chapter-versions",
    summary = "Modifies a chapter version",
    description = r###""###
)]
pub(crate) async fn edit(
    Json(data): Json<ChapterVersionEditRequest>,
    version_service: Data<ChapterVersionActions>,
) -> ApiResult<Json<u8>> {
    version_service
        .edit(&data.id, data.rename, data.update_translate_opts)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "chapter-versions",
    summary = "Deletes a chapter version from a chapter",
    description = r###""###
)]
pub(crate) async fn delete(
    Json(data): Json<ChapterVersionDeleteRequest>,
    chapter_service: Data<ChapterVersionActions>,
) -> ApiResult<Json<u8>> {
    chapter_service
        .delete(&data.chapter_id, &data.version_id)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "chapter-versions",
    summary = "Lists all chapter versions",
    description = r###""###
)]
pub(crate) async fn list(
    Json(data): Json<PaginationRequest>,
    version_service: Data<ChapterVersionActions>,
) -> ApiResult<Json<Vec<VersionInfoResponse>>> {
    version_service.list(data).await.map(Json)
}
