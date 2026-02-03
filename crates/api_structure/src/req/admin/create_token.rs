use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::auth::kind::TokenKind;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct CreateTokenRequest {
    pub user_id: Option<String>,
    pub kind: TokenKind,
}
