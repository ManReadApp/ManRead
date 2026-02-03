use actix_web::web::{Data, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::{jwt::Claim, role::Permission};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::lists::ListDBService};

#[api_operation(
    tag = "list",
    summary = "Lists all lists for the user",
    description = r###""###
)]
pub(crate) async fn exec(
    list_service: Data<ListDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<Vec<String>>> {
    Ok(CreatedJson(list_service.get(&user.id).await?))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
