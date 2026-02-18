use std::{collections::HashMap, sync::Mutex};

use actix_web::{
    dev::ServiceRequest,
    rt::task::spawn_blocking,
    web::{Data, ReqData},
    Error, HttpMessage as _,
};
use actix_web_grants::authorities::AttachAuthorities;

use actix_web_httpauth::extractors::bearer::BearerAuth;

use api_structure::{
    now,
    v1::{Claim, JwtType, Role, Visibility},
};
use bcrypt::DEFAULT_COST;
use db::manga::Manga;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::error::{ApiError, ApiResult};

#[derive(Debug)]
pub struct CryptoService {
    pub secret: Vec<u8>,
    pub claims: Mutex<HashMap<String, Claim>>,
}

impl CryptoService {
    pub fn new(secret: Vec<u8>) -> Self {
        Self {
            secret,
            claims: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn validator(
    req: ServiceRequest,
    cred: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let Some(secret) = req.app_data::<Data<CryptoService>>() else {
        return Err((
            ApiError::write_error("CryptoService is missing").into(),
            req,
        ));
    };
    match secret.get_claim(cred.token()) {
        Ok(v) => {
            {
                if matches!(v.r#type, JwtType::AccessToken) {
                    req.attach(v.role.get_permissions());
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
        let Ok(visibility) = Visibility::try_from(manga.visibility) else {
            return false;
        };
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
    pub async fn hash_password(&self, password: &str) -> ApiResult<String> {
        let password = password.to_string();
        let hashed = spawn_blocking(move || bcrypt::hash(&password, DEFAULT_COST)).await??;

        Ok(hashed)
    }

    /// checks if the password is correct
    pub async fn verify_hash(&self, password: String, hash: String) -> bool {
        spawn_blocking(move || bcrypt::verify(password, &hash).unwrap_or(false))
            .await
            .unwrap_or(false)
    }

    /// Gets the claims from the token
    pub fn get_claim(&self, token: &str) -> ApiResult<Claim> {
        let mut cache = self
            .claims
            .lock()
            .map_err(|_| ApiError::write_error("claim cache lock poisoned"))?;
        if let Some(claim) = cache.get(token).cloned() {
            if claim.exp < now().as_millis() as u64 {
                cache.remove(token);
                return Err(ApiError::ExpiredToken);
            }
            return Ok(claim);
        }
        drop(cache);

        let claim = self.decode_claim(token);
        if let Ok(claim) = &claim {
            let mut cache = self
                .claims
                .lock()
                .map_err(|_| ApiError::write_error("claim cache lock poisoned"))?;
            cache.insert(token.to_string(), claim.clone());
        }
        claim
    }

    /// Internal method to decode the token
    fn decode_claim(&self, token: &str) -> ApiResult<Claim> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let token =
            decode::<Claim>(token, &decoding_key, &Validation::new(Algorithm::HS512))?.claims;

        if token.exp < now().as_millis() as u64 {
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
