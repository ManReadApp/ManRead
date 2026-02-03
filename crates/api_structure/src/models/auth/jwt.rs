use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::role::Role;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claim {
    pub id: String,
    pub role: Role,
    #[serde(rename = "type")]
    pub jwt_type: JwtType,
    pub exp: u128,
}

pub fn now() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

impl Claim {
    pub fn new(uid: String, role: Role, jwt_type: JwtType, dur: Duration) -> Self {
        let expiration = now() + dur;

        Claim {
            id: uid,
            role,
            exp: expiration.as_millis(),
            jwt_type,
        }
    }

    pub fn new_access(uid: String, role: Role) -> Self {
        Self::new(uid, role, JwtType::AccessToken, Duration::from_secs(120)) //2min
    }

    pub fn new_refresh(uid: String, role: Role) -> Self {
        Self::new(
            uid,
            role,
            JwtType::RefreshToken,
            Duration::from_secs(REFRESH_SECS),
        ) // 60days
    }
}

pub const REFRESH_SECS: u64 = 60 * 60 * 24 * 60;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum JwtType {
    AccessToken,
    RefreshToken,
}
