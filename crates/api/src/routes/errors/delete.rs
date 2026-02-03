use actix_web::web::Data;
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::logs::LogDbService};

#[api_operation(
    tag = "errors",
    summary = "Deletes all errors",
    description = r###""###
)]
pub(crate) async fn exec(error_service: Data<LogDbService>) -> ApiResult<CreatedJson<u8>> {
    error_service.clear().await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
