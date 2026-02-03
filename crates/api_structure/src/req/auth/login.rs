use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::auth::login::{LoginWithEmailAndPassword, LoginWithUsernameAndPassword};

#[derive(Deserialize, Serialize, JsonSchema, ApiComponent)]
#[serde(untagged)]
pub enum LoginRequest {
    Username(LoginWithUsernameAndPassword),
    Email(LoginWithEmailAndPassword),
}

impl LoginRequest {
    pub fn password(&self) -> String {
        match self {
            LoginRequest::Username(v) => v.password.clone(),
            LoginRequest::Email(v) => v.password.clone(),
        }
    }
}
