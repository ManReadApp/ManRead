use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{
        ChapterVersion, Claim, MangaReaderRequest, MangaReaderResponse, ReadProgressRequest,
        ReaderPageRequest,
    },
    Permission,
};
use apistos::{api_operation, web::{scope, Scope}};

use crate::{actions::reader::ReaderActions, error::ApiResult};

pub fn register() -> Scope {
    scope("/reader")
        .service(
            apistos::web::resource("/save_progress").route(
                apistos::web::put()
                    .to(save_progress)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/info").route(
                apistos::web::post()
                    .to(info)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/pages_info").route(
                apistos::web::post()
                    .to(pages_info)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
}

#[api_operation(
    tag = "reader",
    summary = "Saves the progress of a chapter",
    description = r###""###
)]
pub(crate) async fn save_progress(
    Json(payload): Json<ReadProgressRequest>,
    reader_service: Data<ReaderActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<u8>> {
    reader_service
        .save_progress(payload.progress, &payload.chapter_id, &user)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "reader",
    summary = "General Reader Info",
    description = r###""###
)]
pub(crate) async fn info(
    Json(payload): Json<MangaReaderRequest>,
    reader_service: Data<ReaderActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<MangaReaderResponse>> {
    reader_service
        .info(&payload.manga_id, payload.chapter_id, &user)
        .await
        .map(Json)
}

#[api_operation(tag = "reader", summary = "creates a list", description = r###""###)]
pub(crate) async fn pages_info(
    Json(payload): Json<ReaderPageRequest>,
    reader_service: Data<ReaderActions>,
) -> ApiResult<Json<ChapterVersion>> {
    reader_service
        .pages(&payload.chapter_version_id)
        .await
        .map(Json)
}
