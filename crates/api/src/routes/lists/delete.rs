use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::list::add::DeleteListRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::lists::ListDBService};

#[api_operation(tag = "list", summary = "deletes a list", description = r###""###)]
pub(crate) async fn exec(
    Json(payload): Json<DeleteListRequest>,
    list_service: Data<ListDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service.delete(&payload.name, &user.id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
