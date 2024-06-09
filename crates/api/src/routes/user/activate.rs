use crate::errors::ApiResult;
use crate::services::crypto_service::CryptoService;
use crate::services::db::auth_tokens::AuthTokenDBService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::protect;
use api_structure::auth::activate::ActivateRequest;
use api_structure::auth::jwt::{Claim, JWTs};
use api_structure::error::{ApiErr, ApiErrorType};

#[post("/auth/activate")]
#[protect(
    any("api_structure::auth::role::Role::NotVerified"),
    ty = "api_structure::auth::role::Role"
)]
async fn activate(
    claim: ReqData<Claim>,
    Json(data): Json<ActivateRequest>,
    user: Data<UserDBService>,
    crypto: Data<CryptoService>,
    activation: Data<AuthTokenDBService>,
) -> ApiResult<Json<JWTs>> {
    let find = activation.check(&data.key).await?;
    if let Some(v) = &find.data.user {
        if v.thing.id().to_string() != claim.id {
            return Err(ApiErr {
                message: Some("Not valid token".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }
            .into());
        }
    }
    let kind = find.data.get_kind();
    if kind.single {
        find.delete_s(&*activation.conn).await?;
    }

    user.set_role(claim.id.as_str(), kind.kind).await?;

    Ok(Json(JWTs {
        access_token: crypto.encode_claim(&Claim::new_access(claim.id.clone(), kind.kind)?)?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(claim.id.clone(), kind.kind)?)?,
    }))
}
