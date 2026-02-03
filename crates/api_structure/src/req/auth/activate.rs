use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct ActivateRequest {
    pub key: String,
}
