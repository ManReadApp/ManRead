use actix_web::web::{Data, Json};
use api_structure::{
    models::auth::{kind::TokenKind, role::Role},
    req::auth::reset_password::RequestResetPasswordRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{auth::AuthTokenDBService, user::UserDBService},
};

#[api_operation(
    tag = "user",
    summary = "Requests the Reset of the password",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<RequestResetPasswordRequest>,
    user_service: Data<UserDBService>,
    token_service: Data<AuthTokenDBService>,
) -> ApiResult<CreatedJson<u8>> {
    let user = match data.email {
        true => user_service.get_by_mail(&data.ident).await,
        false => user_service.get_by_name(&data.ident).await,
    }?;
    token_service
        .create(
            Some(user.id.id().to_string()),
            TokenKind {
                single: true,
                kind: Role::NotVerified,
            },
        )
        .await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/reset-password").route(apistos::web::post().to(exec))
}
