use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JWTsResponse {
    pub access_token: String,
    pub refresh_token: String,
}
