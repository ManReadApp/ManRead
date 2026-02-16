use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{v1::KindListResponse, Permission};
use apistos::{api_operation, web::scope};

use crate::{actions::kind::KindActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    scope("/kind").service(
        apistos::web::resource("/list").route(
            apistos::web::post()
                .to(list)
                .guard(AuthorityGuard::new(Permission::Read)),
        ),
    )
}

#[api_operation(
    tag = "kind",
    summary = "Lists all manga kinds",
    description = r###""###
)]
pub(crate) async fn list(list_service: Data<KindActions>) -> ApiResult<Json<KindListResponse>> {
    Ok(Json(KindListResponse {
        items: list_service.list().await?,
    }))
}
