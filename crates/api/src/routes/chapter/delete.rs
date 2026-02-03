use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::IdRequest};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::chapter::ChapterDBService};

#[api_operation(
    tag = "chapter",
    summary = "Deletes a chapter",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterDBService>,
) -> ApiResult<CreatedJson<u8>> {
    chapter_service.delete(&data.id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::RequestDelete)),
    )
}
