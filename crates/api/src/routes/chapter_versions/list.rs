use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission, req::PaginationRequest,
    resp::version::list::VersionInfoResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::version::VersionDBService};

#[api_operation(
    tag = "chapter-versions",
    summary = "Lists all chapter versions",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<PaginationRequest>,
    version_service: Data<VersionDBService>,
) -> ApiResult<CreatedJson<Vec<VersionInfoResponse>>> {
    let versions = version_service.list(data.page, data.limit).await?;
    Ok(CreatedJson(
        versions
            .into_iter()
            .map(|v| VersionInfoResponse {
                id: v.id.id().to_string(),
                name: v.data.name,
                translate_opts: v.data.translate_opts,
            })
            .collect(),
    ))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
