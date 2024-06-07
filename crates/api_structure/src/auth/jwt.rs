use crate::auth::role::Role;
use crate::error::ApiErr;
use crate::{now_timestamp, RequestImpl};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claim {
    pub id: String,
    pub role: Role,
    #[serde(rename = "type")]
    pub jwt_type: JwtType,
    pub exp: u128,
}

impl Claim {
    pub fn new(uid: String, role: Role, jwt_type: JwtType, dur: Duration) -> Result<Self, ApiErr> {
        let expiration = now_timestamp()? + dur;

        Ok(Claim {
            id: uid,
            role,
            exp: expiration.as_millis(),
            jwt_type,
        })
    }

    pub fn new_access(uid: String, role: Role) -> Result<Self, ApiErr> {
        Self::new(uid, role, JwtType::AccessToken, Duration::from_secs(120)) //2min
    }

    pub fn new_refresh(uid: String, role: Role) -> Result<Self, ApiErr> {
        Self::new(
            uid,
            role,
            JwtType::RefreshToken,
            Duration::from_secs(60 * 60 * 24 * 60),
        ) // 60days
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum JwtType {
    AccessToken,
    RefreshToken,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Response
pub struct JWTs {
    pub access_token: String,
    pub refresh_token: String,
}

impl RequestImpl for JWTs {
    const ROUTE: &'static str = "refresh";
    const AUTH: bool = true;
}
