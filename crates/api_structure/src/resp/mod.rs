use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::manga::external_search::ValidSearches;

pub mod auth;
pub mod chapter;
pub mod external;
pub mod manga;
pub mod reader;
pub mod version;

pub type ByteResponse = Vec<u8>;
pub type NoResponse = Vec<u8>;

pub type FontsResponse = Vec<String>;
pub type AvailableExternalSitesResponse = HashMap<String, ValidSearches>;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct ErrorResponse {
    pub message: String,
    pub timestamp: u128,
}
