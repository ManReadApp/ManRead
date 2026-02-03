use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::auth::kind::TokenKind;

#[derive(Debug, Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct JWTsResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct TokenInfo {
    pub token_id: String,
    pub user_id: Option<String>,
    pub kind: TokenKind,
}
