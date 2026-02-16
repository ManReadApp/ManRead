use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{Claim, IdRequest, PaginationRequest, SearchRequest, SimpleUser, UpdateUserRequest, User},
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{actions::user::UserActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/user")
        .service(
            apistos::web::resource("/delete").route(
                apistos::web::delete()
                    .to(delete)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/edit").route(
                apistos::web::put()
                    .to(edit)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/info").route(
                apistos::web::post()
                    .to(info)
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
            apistos::web::resource("/search").route(
                apistos::web::post()
                    .to(search)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
}

#[api_operation(tag = "user", summary = "Deletes a user", description = r###""###)]
pub(crate) async fn delete(
    Json(data): Json<IdRequest>,
    user_service: Data<UserActions>,
) -> ApiResult<CreatedJson<u8>> {
    user_service.delete(&data.id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "user",
    summary = "Edits a user",
    description = r###"Multiple edits are possible at once, but shouldnt be used. It will stop in the middle of the changes if there is a reason to stop. E.g. wrong password"###
)]
pub(crate) async fn edit(
    Json(data): Json<UpdateUserRequest>,
    claim: ReqData<Claim>,
    user_service: Data<UserActions>,
) -> ApiResult<CreatedJson<u8>> {
    user_service.edit(data, &claim).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "user",
    summary = "Displays info about the user",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn info(
    Json(data): Json<IdRequest>,
    user_service: Data<UserActions>,
) -> ApiResult<Json<User>> {
    user_service.info(&data.id).await.map(Json)
}

#[api_operation(tag = "user", summary = "Lists all users", description = r###""###)]
pub(crate) async fn list(
    Json(data): Json<PaginationRequest>,
    user_service: Data<UserActions>,
) -> ApiResult<Json<Vec<SimpleUser>>> {
    user_service.list(data).await.map(Json)
}

#[api_operation(
    tag = "user",
    summary = "Searches the users",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn search(
    Json(data): Json<SearchRequest>,
    user_service: Data<UserActions>,
) -> ApiResult<Json<Vec<SimpleUser>>> {
    user_service.search(data).await.map(Json)
}
