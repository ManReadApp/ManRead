use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::version::ChapterVersionEditRequest};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::version::VersionDBService};

#[api_operation(
    tag = "chapter-versions",
    summary = "Modifies a chapter version",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ChapterVersionEditRequest>,
    version_service: Data<VersionDBService>,
) -> ApiResult<CreatedJson<u8>> {
    if let Some(name) = data.rename {
        version_service.rename(&data.id, name).await?;
    }
    if let Some(translate_opts) = data.update_translate_opts {
        version_service
            .update_translate_opts(&data.id, translate_opts)
            .await?;
    }
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/edit").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
