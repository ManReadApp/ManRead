use actix_web::web::{self, Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{
        AddListRequest, AddMangaToListRequest, Claim, DeleteListRequest, RemoveMangaToListRequest,
    },
    Permission,
};
use apistos::{
    actix::CreatedJson,
    api_operation,
    web::{scope, Scope},
};

use crate::{actions::lists::ListActions, error::ApiResult};

pub fn register() -> Scope {
    scope("/lists")
        .service(
            apistos::web::resource("/add").route(
                apistos::web::put()
                    .to(add)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/delete").route(
                apistos::web::delete()
                    .to(delete)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/list").route(
                apistos::web::post()
                    .to(list)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            scope("/{list}")
                .service(
                    apistos::web::resource("/add").route(
                        apistos::web::put()
                            .to(add_to_list)
                            .guard(AuthorityGuard::new(Permission::Read)),
                    ),
                )
                .service(
                    apistos::web::resource("/delete").route(
                        apistos::web::delete()
                            .to(remove_from_list)
                            .guard(AuthorityGuard::new(Permission::Read)),
                    ),
                ),
        )
}

#[api_operation(
    tag = "list-item",
    summary = "Adds a manga to a list",
    description = r###""###
)]
pub(crate) async fn add_to_list(
    list: web::Path<String>,
    Json(payload): Json<AddMangaToListRequest>,
    list_service: Data<ListActions>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service
        .add_to_list(&list, &payload.manga_id, &user)
        .await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "list-item",
    summary = "Deletes a manga from a list",
    description = r###""###
)]
pub(crate) async fn remove_from_list(
    list: web::Path<String>,
    Json(payload): Json<RemoveMangaToListRequest>,
    list_service: Data<ListActions>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service
        .remove_from_list(&list, &payload.manga_id, &user)
        .await?;
    Ok(CreatedJson(0))
}

#[api_operation(tag = "list", summary = "creates a list", description = r###""###)]
pub(crate) async fn add(
    Json(payload): Json<AddListRequest>,
    list_service: Data<ListActions>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service.add(&payload.name, &user).await?;
    Ok(CreatedJson(0))
}

#[api_operation(tag = "list", summary = "deletes a list", description = r###""###)]
pub(crate) async fn delete(
    Json(payload): Json<DeleteListRequest>,
    list_service: Data<ListActions>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    list_service.remove(&payload.name, &user).await?;
    Ok(CreatedJson(0))
}
#[api_operation(
    tag = "list",
    summary = "Lists all lists for the user",
    description = r###""###
)]
pub(crate) async fn list(
    list_service: Data<ListActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<Vec<String>>> {
    list_service.list(&user).await.map(Json)
}
