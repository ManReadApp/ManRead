use std::collections::HashMap;
use std::sync::Mutex;

use crate::errors::{ApiError, ApiResult};
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::models::auth::jwt::Claim;
use api_structure::now_timestamp;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug)]
pub struct CryptoService {
    pub secret: Vec<u8>,
    pub claims: Mutex<HashMap<String, Claim>>,
}

impl CryptoService {
    pub fn hash_password(&self, password: &str) -> ApiResult<String> {
        let hashed = hash(password, DEFAULT_COST)?;
        Ok(hashed)
    }

    pub fn verify_hash(&self, password: String, hash: String) -> bool {
        verify(password, &hash).unwrap_or(false)
    }

    pub fn get_claim(&self, token: &str) -> ApiResult<Claim> {
        if let Some(v) = self.claims.lock()?.get(token) {
            if v.exp < now_timestamp()?.as_millis() {
                self.claims.lock()?.remove(token);
                return Err(ApiError::expired_token_error("token expired"));
            }
            return Ok(v.clone());
        }
        let claim = self.decode_claim(token);
        if let Ok(claim) = &claim {
            self.claims.lock()?.insert(token.to_string(), claim.clone());
        }
        claim
    }

    pub fn decode_claim(&self, token: &str) -> ApiResult<Claim> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let token = match decode::<Claim>(token, &decoding_key, &Validation::new(Algorithm::HS512))
        {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(ApiError::invalid_token_error("Invalid token", e)),
        }?;
        if token.exp < now_timestamp()?.as_millis() {
            Err(ApiError::expired_token_error("Token expired"))
        } else {
            Ok(token)
        }
    }

    pub fn encode_claim(&self, claim: &Claim) -> ApiResult<String> {
        let header = Header::new(Algorithm::HS512);
        jsonwebtoken::encode(&header, claim, &EncodingKey::from_secret(&self.secret)).map_err(|e| {
            ApiErr {
                message: Some("couldnt create token".to_string()),
                cause: Some(e.to_string()),
                err_type: ApiErrorType::InternalError,
            }
            .into()
        })
    }
}
