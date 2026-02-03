use actix_web::web::{self, Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::list::add::AddMangaToListRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{lists::ListDBService, manga::MangaDBService},
};

#[api_operation(
    tag = "list-item",
    summary = "Adds a manga to a list",
    description = r###""###
)]
pub(crate) async fn exec(
    list: web::Path<String>,
    Json(payload): Json<AddMangaToListRequest>,
    list_service: Data<ListDBService>,
    manga_service: Data<MangaDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service.exists(&payload.manga_id).await?;
    list_service
        .add_manga(&list, &user.id, &payload.manga_id)
        .await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/add").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
