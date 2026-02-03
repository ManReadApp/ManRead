use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::chapter::progress::ReadProgressRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{chapter::ChapterDBService, progress::UserProgressDBService},
};

#[api_operation(
    tag = "chapter",
    summary = "Saves the progress of a chapter",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ReadProgressRequest>,
    progress_service: Data<UserProgressDBService>,
    chapter_service: Data<ChapterDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    let progress = data.progress.clamp(0.0, 1.0);
    let manga_id = chapter_service.get_manga_id(&data.chapter_id).await?;
    progress_service
        .update(&user.id, &manga_id, &data.chapter_id, progress)
        .await?;
    if progress >= 0.95 {
        let _ = progress_service
            .load_next_chapter(&user.id, &manga_id, &data.chapter_id)
            .await;
    }
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/save_progress").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
