use crate::errors::ApiResult;
use crate::services::crypto_service::CryptoService;
use crate::services::db::auth_tokens::{AuthToken, AuthTokenDBService};
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json};
use api_structure::auth::jwt::{Claim, JWTs};
use api_structure::auth::reset_password::{RequestResetPasswordRequest, ResetPasswordRequest};
use api_structure::auth::role::Role;
use api_structure::error::{ApiErr, ApiErrorType};
use surrealdb_extras::SurrealTableInfo;

#[post("/request_reset_password")]
async fn request_reset_password(
    Json(data): Json<RequestResetPasswordRequest>,
    activation: Data<AuthTokenDBService>,
    user: Data<UserDBService>,
) -> ApiResult<Json<()>> {
    let id = user.get_id(&data.ident, data.email).await?;
    AuthToken::new_forgot(id).add_i(&*activation.conn).await?;
    Ok(Json(()))
}
#[post("/reset_password")]
async fn reset_password(
    Json(data): Json<ResetPasswordRequest>,
    user: Data<UserDBService>,
    crypto: Data<CryptoService>,
    activation: Data<AuthTokenDBService>,
) -> ApiResult<Json<JWTs>> {
    let find = activation.check(&data.key).await?;
    let id = user.get_id(&data.ident, data.email).await?;
    if let Some(v) = &find.data.user {
        if v.thing.id().to_string() != id {
            return Err(ApiErr {
                message: Some("Not valid token".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }
            .into());
        }
    }
    let kind = find.data.get_kind();
    if kind.kind != Role::NotVerified {
        return Err(ApiErr {
            message: Some("Not valid token".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into());
    }

    if kind.single {
        find.delete_s(&*activation.conn).await?;
    }
    let hash = crypto.hash_password(&data.password)?;
    user.set_password(id.as_str(), hash).await?;

    Ok(Json(JWTs {
        access_token: crypto.encode_claim(&Claim::new_access(id.clone(), kind.kind)?)?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(id.clone(), kind.kind)?)?,
    }))
}
