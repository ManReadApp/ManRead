use crate::errors::ApiResult;
use crate::services::crypto_service::CryptoService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json, ReqData};
use api_structure::models::auth::jwt::Claim;
use api_structure::resp::auth::JWTsResponse;

#[post("/refresh")]
async fn refresh_(
    claim: ReqData<Claim>,
    db: Data<UserDBService>,
    crypto: Data<CryptoService>,
) -> ApiResult<Json<JWTsResponse>> {
    let role = db.get_role(claim.id.as_str()).await?;
    Ok(Json(JWTsResponse {
        access_token: crypto.encode_claim(&Claim::new_access(claim.id.clone(), role)?)?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(claim.id.clone(), role)?)?,
    }))
}
