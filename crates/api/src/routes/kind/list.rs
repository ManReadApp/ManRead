use actix_web::web::Data;
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;

use crate::{error::ApiResult, models::kind::KindDBService};

#[derive(serde::Serialize, serde::Deserialize, JsonSchema, ApiComponent)]
pub struct ResponseTemp {
    pub items: Vec<String>,
}
#[api_operation(
    tag = "kind",
    summary = "Lists all manga kinds",
    description = r###""###
)]
pub(crate) async fn exec(
    list_service: Data<KindDBService>,
) -> ApiResult<CreatedJson<ResponseTemp>> {
    Ok(CreatedJson(ResponseTemp {
        items: list_service.all().await?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
