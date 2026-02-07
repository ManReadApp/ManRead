use actix_web::web::{Data, Json};
use api_structure::{
    req::{admin::create_token::CreateTokenRequest, IdRequest, PaginationRequest},
    resp::auth::TokenInfo,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::actions::token::TokenAction;

#[api_operation(
    tag = "admin",
    summary = "Lists the activation codes",
    description = r###""###
)]
async fn list(
    Json(data): Json<PaginationRequest>,
    token_service: Data<TokenAction>,
) -> ApiResult<CreatedJson<Vec<TokenInfo>>> {
    Ok(CreatedJson(
        token_service
            .list_tokens(data.page, data.limit)
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

#[api_operation(
    tag = "admin",
    summary = "Deletes an activation code",
    description = r###""###
)]
pub(crate) async fn delete(
    Json(data): Json<IdRequest>,
    token_service: Data<TokenAction>,
) -> ApiResult<CreatedJson<u8>> {
    token_service.delete_token(&data.id).await?;
    Ok(CreatedJson(200))
}

#[api_operation(
    tag = "admin",
    summary = "Creates a activation code",
    description = r###""###
)]
pub(crate) async fn create(
    Json(data): Json<CreateTokenRequest>,
    token_service: Data<TokenAction>,
) -> ApiResult<CreatedJson<u8>> {
    token_service.create_token(data.user_id, data.kind).await?;
    Ok(CreatedJson(200))
}

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/admin").service(
        apistos::web::scope("/verify")
            .service(apistos::web::resource("/create").route(apistos::web::put().to(create)))
            .service(apistos::web::resource("/delete").route(apistos::web::delete().to(delete)))
            .service(apistos::web::resource("/list").route(apistos::web::post().to(list))),
    )
}
