use actix_web::web::{Data, Json};
use api_structure::{req::PaginationRequest, resp::auth::TokenInfo};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::auth::AuthTokenDBService};
#[api_operation(
    tag = "admin",
    summary = "Lists the activation codes",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<PaginationRequest>,
    token_service: Data<AuthTokenDBService>,
) -> ApiResult<CreatedJson<Vec<TokenInfo>>> {
    Ok(CreatedJson(
        token_service
            .list(data.page, data.limit)
            .await?
            .into_iter()
            .map(|v| TokenInfo {
                token_id: v.id.id().to_string(),
                kind: v.data.get_kind(),
                user_id: v.data.user.map(|v| v.thing.id().to_string()),
            })
            .collect(),
    ))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/list").route(apistos::web::post().to(exec))
}
