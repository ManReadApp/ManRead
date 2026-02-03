use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::auth::UserSearchRequest};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;

use crate::{error::ApiResult, models::tag::TagDBService};

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
    Json(data): Json<UserSearchRequest>,
    list_service: Data<TagDBService>,
) -> ApiResult<CreatedJson<Vec<api_structure::models::manga::tag::Tag>>> {
    Ok(CreatedJson(
        list_service
            .search(&data.query, data.page, data.limit)
            .await?,
    ))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/search").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
