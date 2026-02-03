use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::list::add::AddListRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::lists::ListDBService};

#[api_operation(tag = "list", summary = "creates a list", description = r###""###)]
pub(crate) async fn exec(
    Json(payload): Json<AddListRequest>,
    list_service: Data<ListDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service.add(&payload.name, &user.id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/add").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
