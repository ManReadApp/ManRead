use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{gender::Gender, role::Permission},
    req::PaginationRequest,
};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{error::ApiResult, models::user::UserDBService};
#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct SimpleUser {
    pub id: String,
    pub names: Vec<String>,
    pub icon_ext: Option<String>,
    pub gender: Gender,
}

#[api_operation(tag = "user", summary = "Lists all users", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<PaginationRequest>,
    user_service: Data<UserDBService>,
) -> ApiResult<CreatedJson<Vec<SimpleUser>>> {
    let items = user_service.list(data.page, data.limit).await?;
    Ok(CreatedJson(
        items
            .into_iter()
            .map(|v| SimpleUser {
                id: v.id.id().to_string(),
                names: v.data.names,
                icon_ext: v.data.icon_ext,
                gender: Gender::from(v.data.gender as usize),
            })
            .collect::<Vec<_>>(),
    ))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
