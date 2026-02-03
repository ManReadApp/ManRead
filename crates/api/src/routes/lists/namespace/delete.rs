use actix_web::web::{self, Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::list::add::RemoveMangaToListRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::lists::ListDBService};

#[api_operation(
    tag = "list-item",
    summary = "Deletes a manga from a list",
    description = r###""###
)]
pub(crate) async fn exec(
    list: web::Path<String>,
    Json(payload): Json<RemoveMangaToListRequest>,
    list_service: Data<ListDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service
        .remove_manga(&list, &user.id, &payload.manga_id)
        .await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
