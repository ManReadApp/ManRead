use actix_web::web::Data;
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, resp::ErrorResponse};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::logs::LogDbService};

#[api_operation(
    tag = "errors",
    summary = "Lists all errors from services",
    description = r###""###
)]
pub(crate) async fn exec(
    error_serrice: Data<LogDbService>,
) -> ApiResult<CreatedJson<Vec<ErrorResponse>>> {
    let versions = error_serrice.list().await?;
    Ok(CreatedJson(
        versions
            .into_iter()
            .map(|v| ErrorResponse {
                message: v.message,
                timestamp: v.created_at.into_inner().timestamp() as u128,
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
