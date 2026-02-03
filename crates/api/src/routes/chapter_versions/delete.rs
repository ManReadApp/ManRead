use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission, req::chapter::delete_version::ChapterVersionDeleteRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::chapter::ChapterDBService};

#[api_operation(
    tag = "chapter-versions",
    summary = "Deletes a chapter version from a chapter",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ChapterVersionDeleteRequest>,
    chapter_service: Data<ChapterDBService>,
) -> ApiResult<CreatedJson<u8>> {
    chapter_service
        .delete_version(&data.chapter_id, &data.version_id)
        .await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
