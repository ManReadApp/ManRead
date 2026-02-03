use actix_web::web::{Data, Json};
use api_structure::{
    models::auth::{jwt::Claim, role::Role},
    req::auth::login::LoginRequest,
    resp::auth::JWTsResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    models::user::UserDBService,
    services::auth::CryptoService,
};

#[api_operation(tag = "auth", summary = "Logs in the user", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<LoginRequest>,
    user_service: Data<UserDBService>,
    crypto_service: Data<CryptoService>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    let user = match &data {
        LoginRequest::Username(l) => user_service.get_by_name(&l.username).await,
        LoginRequest::Email(l) => user_service.get_by_mail(&l.email).await,
    }?;
    let valid = crypto_service.verify_hash(data.password(), user.data.password);
    if !valid {
        return Err(ApiError::PasswordIncorrect);
    }
    Ok(CreatedJson(JWTsResponse {
        access_token: crypto_service.encode_claim(&Claim::new_access(
            user.id.id().to_string(),
            Role::from(user.data.role),
        ))?,
        refresh_token: crypto_service.encode_claim(&Claim::new_refresh(
            user.id.id().to_string(),
            Role::from(user.data.role),
        ))?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/sign-in").route(apistos::web::post().to(exec))
}
