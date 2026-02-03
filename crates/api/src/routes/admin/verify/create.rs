use actix_web::web::{Data, Json};
use api_structure::req::admin::create_token::CreateTokenRequest;
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::auth::AuthTokenDBService};

#[api_operation(
    tag = "admin",
    summary = "Creates a activation code",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<CreateTokenRequest>,
    token_service: Data<AuthTokenDBService>,
) -> ApiResult<CreatedJson<u8>> {
    token_service.create(data.user_id, data.kind).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/create").route(apistos::web::put().to(exec))
}
