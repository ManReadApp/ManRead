use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod admin;
pub mod auth;
pub mod chapter;
pub mod fonts;
pub mod list;
pub mod manga;
pub mod reader;
pub mod version;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct IdRequest {
    pub id: String,
}

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct PaginationRequest {
    pub page: u32,
    pub limit: u32,
}
