use actix_web::web::{Data, Json};
use api_structure::{
    models::auth::{jwt::Claim, role::Role},
    req::auth::reset_password::ResetPasswordRequest,
    resp::auth::JWTsResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::{auth::AuthTokenDBService, user::UserDBService},
    services::auth::CryptoService,
};

#[api_operation(
    tag = "user",
    summary = "Updates the forgotten password",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ResetPasswordRequest>,
    user_service: Data<UserDBService>,
    token_service: Data<AuthTokenDBService>,
    crypto_service: Data<CryptoService>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    let user = match data.email {
        true => user_service.get_by_mail(&data.ident).await,
        false => user_service.get_by_name(&data.ident).await,
    }?;

    let token = token_service.find(&data.key).await?;
    if let Some(max_age) = token.data.active_until_timestamp {
        if max_age < chrono::Utc::now().timestamp() as u64 {
            return Err(ApiError::ExpiredToken);
        }
    }
    if token.data.user.as_ref().map(|v| v.id()) == Some(user.id.id()) {
        match token.data.get_kind().kind {
            Role::NotVerified => {
                let hash = crypto_service.hash_password(&data.password)?;
                user_service
                    .set_password(&user.id.id().to_string(), hash)
                    .await?;
                let kind = token.data.get_kind();
                let role = kind.kind;
                if kind.single {
                    token.delete_s(&*DB).await?;
                }
                return Ok(CreatedJson(JWTsResponse {
                    access_token: crypto_service
                        .encode_claim(&Claim::new_access(user.id.id().to_string(), role))?,
                    refresh_token: crypto_service
                        .encode_claim(&Claim::new_refresh(user.id.id().to_string(), role))?,
                }));
            }
            _ => {}
        }
    }

    return Err(ApiError::WrongResetToken);
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/verify-reset-password").route(apistos::web::put().to(exec))
}
