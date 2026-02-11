use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod external;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct ErrorResponse {
    pub message: String,
    pub timestamp: u128,
}
