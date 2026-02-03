use std::time::Duration;

use actix_web::web::{Data, Json};
use api_structure::{
    models::auth::jwt::{Claim, REFRESH_SECS},
    req::auth::TokenRefreshRequest,
    resp::auth::JWTsResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    models::user::UserDBService,
    services::auth::CryptoService,
};

#[api_operation(tag = "auth", summary = "Refreshes the token", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<TokenRefreshRequest>,
    user_service: Data<UserDBService>,
    crypto: Data<CryptoService>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    let claim = crypto.get_claim(&data.refresh_token)?;
    let (role, generated) = user_service
        .get_role_and_generated(claim.id.as_str())
        .await?;
    if generated > claim.exp - Duration::from_secs(REFRESH_SECS).as_millis() {
        return Err(ApiError::ExpiredToken);
    }
    Ok(CreatedJson(JWTsResponse {
        access_token: crypto.encode_claim(&Claim::new_access(claim.id.clone(), role))?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(claim.id.clone(), role))?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/refresh").route(apistos::web::post().to(exec))
}
