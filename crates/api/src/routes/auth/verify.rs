use actix_web::web::{Data, Json, ReqData};
use api_structure::{
    models::auth::jwt::Claim, req::auth::activate::ActivateRequest, resp::auth::JWTsResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::{auth::AuthTokenDBService, user::UserDBService},
    services::auth::CryptoService,
};

#[api_operation(tag = "auth", summary = "Verifies the user", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<ActivateRequest>,
    activation_service: Data<AuthTokenDBService>,
    user_service: Data<UserDBService>,
    crypto_service: Data<CryptoService>,
    claim: ReqData<Claim>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    let find = activation_service.find(&data.key).await?;
    if let Some(v) = &find.data.user {
        if v.id().to_string() != claim.id {
            return Err(ApiError::InvalidActivationToken);
        }
    }
    if let Some(max_age) = find.data.active_until_timestamp {
        if max_age < chrono::Utc::now().timestamp() as u64 {
            return Err(ApiError::ExpiredToken);
        }
    }
    let kind = find.data.get_kind();
    if kind.single {
        find.delete_s(&*DB).await?;
    }

    user_service.set_role(claim.id.as_str(), kind.kind).await?;

    Ok(CreatedJson(JWTsResponse {
        access_token: crypto_service
            .encode_claim(&Claim::new_access(claim.id.clone(), kind.kind))?,
        refresh_token: crypto_service
            .encode_claim(&Claim::new_refresh(claim.id.clone(), kind.kind))?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/verify-account").route(apistos::web::put().to(exec))
}
