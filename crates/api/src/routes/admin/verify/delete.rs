use actix_web::web::{Data, Json};
use api_structure::req::IdRequest;
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::auth::AuthTokenDBService};

#[api_operation(
    tag = "admin",
    summary = "Deletes an activation code",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    token_service: Data<AuthTokenDBService>,
) -> ApiResult<CreatedJson<u8>> {
    token_service.delete(&data.id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(apistos::web::delete().to(exec))
}
