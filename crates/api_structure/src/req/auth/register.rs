use crate::models::auth::gender::Gender;
use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, ApiComponent, JsonSchema)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub birthdate: DateTime<Utc>,
    pub gender: Gender,
    pub icon_temp_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ApiComponent, JsonSchema)]
pub struct UpdateUserRequest {
    pub name: Option<Vec<String>>,
    pub password: Option<(String, String)>,
    pub icon_temp_name: Option<String>,
    pub description: Option<String>,
    pub links: Option<Vec<String>>,
    pub thumbnail: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AdminCreateUserRequest {
    pub name: String,
}
