use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use actix_web::{
    dev::ServiceRequest,
    web::{Data, ReqData},
    Error, HttpMessage as _,
};
use actix_web_grants::authorities::AttachAuthorities;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use api_structure::models::{
    auth::{
        jwt::{Claim, JwtType},
        role::Role,
    },
    manga::visiblity::Visibility,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::{
    error::{ApiError, ApiResult},
    models::manga::Manga,
};

#[derive(Debug)]
pub struct CryptoService {
    pub secret: Vec<u8>,
    pub claims: Mutex<HashMap<String, Claim>>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub async fn validator(
    req: ServiceRequest,
    cred: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let secret = req
        .app_data::<Data<CryptoService>>()
        .expect("CryptoService is missing");
    match secret.get_claim(cred.token()) {
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

impl CryptoService {
    /// Visibility guard
    pub fn can_access(&self, user: ReqData<Claim>, manga: Manga) -> bool {
        let visibility = Visibility::try_from(manga.visibility).unwrap();
        match visibility {
            Visibility::Visible => true,
            Visibility::Hidden => {
                matches!(user.role, Role::Admin | Role::CoAdmin | Role::Moderator)
                    || (user.role == Role::Author && manga.uploader.id().to_string() == user.id)
            }
            Visibility::AdminReview => {
                matches!(user.role, Role::Admin | Role::CoAdmin | Role::Moderator)
            }
        }
    }

    /// creates a password hash
    pub fn hash_password(&self, password: &str) -> ApiResult<String> {
        let hashed = hash(password, DEFAULT_COST)?;
        Ok(hashed)
    }

    /// checks if the password is correct
    pub fn verify_hash(&self, password: String, hash: String) -> bool {
        verify(password, &hash).unwrap_or(false)
    }

    /// Gets the claims from the token
    pub fn get_claim(&self, token: &str) -> ApiResult<Claim> {
        if let Some(v) = self.claims.lock().unwrap().get(token) {
            if v.exp < now_ms() {
                self.claims.lock().unwrap().remove(token);
                return Err(ApiError::ExpiredToken);
            }
            return Ok(v.clone());
        }
        let claim = self.decode_claim(token);
        if let Ok(claim) = &claim {
            self.claims
                .lock()
                .unwrap()
                .insert(token.to_string(), claim.clone());
        }
        claim
    }

    /// Internal method to decode the token
    fn decode_claim(&self, token: &str) -> ApiResult<Claim> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let token =
            decode::<Claim>(token, &decoding_key, &Validation::new(Algorithm::HS512))?.claims;

        if token.exp < now_ms() {
            Err(ApiError::ExpiredToken)
        } else {
            Ok(token)
        }
    }

    /// Generates token from Claim
    pub fn encode_claim(&self, claim: &Claim) -> ApiResult<String> {
        let header = Header::new(Algorithm::HS512);
        jsonwebtoken::encode(&header, claim, &EncodingKey::from_secret(&self.secret))
            .map_err(ApiError::generate_jwt)
    }
}
