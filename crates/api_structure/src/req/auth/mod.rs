use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod activate;
pub mod login;
pub mod register;
pub mod reset_password;

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct TokenRefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct UserSearchRequest {
    pub query: String,
    pub page: u32,
    pub limit: u32,
}
