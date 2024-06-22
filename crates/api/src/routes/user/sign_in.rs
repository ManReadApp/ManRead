use crate::errors::ApiResult;
use crate::services::crypto_service::CryptoService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json};
use api_structure::auth::jwt::{Claim, JWTs};
use api_structure::auth::login::LoginRequest;
use api_structure::auth::role::Role;
use api_structure::error::{ApiErr, ApiErrorType};

#[post("/sign_in")]
async fn login(
    Json(data): Json<LoginRequest>,
    user: Data<UserDBService>,
    crypto: Data<CryptoService>,
) -> ApiResult<Json<JWTs>> {
    let (item, password) = match data {
        LoginRequest::Username(v) => (user.login_data(&v.username, false).await, v.password),
        LoginRequest::Email(v) => (user.login_data(&v.email, true).await, v.password),
    };
    let item = item?;
    let valid = crypto.verify_hash(password, item.data.password);
    if !valid {
        return Err(ApiErr {
            message: Some("Password is incorrect".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into());
    }
    Ok(Json(JWTs {
        access_token: crypto.encode_claim(&Claim::new_access(
            item.id.id().to_string(),
            Role::from(item.data.role),
        )?)?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(
            item.id.id().to_string(),
            Role::from(item.data.role),
        )?)?,
    }))
}
