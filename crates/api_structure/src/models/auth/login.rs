use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema, ApiComponent)]
pub struct LoginWithUsernameAndPassword {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, JsonSchema, ApiComponent)]
pub struct LoginWithEmailAndPassword {
    pub email: String,
    pub password: String,
}
