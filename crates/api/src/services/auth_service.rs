use crate::services::crypto_service::CryptoService;
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use actix_web_grants::authorities::AttachAuthorities;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use api_structure::models::auth::jwt::JwtType;

pub async fn validator(
    req: ServiceRequest,
    cred: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let secret = req
        .app_data::<Data<CryptoService>>()
        .expect("CryptoService is missing");
    match secret.decode_claim(cred.token()) {
        Ok(v) => {
            {
                if matches!(v.jwt_type, JwtType::AccessToken) {
                    req.attach(vec![v.role]);
                }
                let mut ext = req.extensions_mut();
                ext.insert(v);
            }
            Ok(req)
        }
        Err(e) => Err((e.into(), req)),
    }
}
