use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::manga::search::Array;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, ApiComponent, JsonSchema)]
pub struct SearchRequest {
    pub order: String,
    pub desc: bool,
    pub limit: u32,
    pub page: u32,
    pub query: Array,
}
