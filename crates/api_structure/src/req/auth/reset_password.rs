use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct ResetPasswordRequest {
    pub ident: String,
    pub email: bool,
    pub key: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct RequestResetPasswordRequest {
    pub ident: String,
    pub email: bool,
}
