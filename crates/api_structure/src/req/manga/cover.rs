use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema, ApiComponent)]
pub struct MangaCoverRequest {
    pub manga_id: String,
    pub file_ext: String,
}
