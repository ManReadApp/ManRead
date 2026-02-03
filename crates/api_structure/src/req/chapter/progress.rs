use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, JsonSchema, ApiComponent)]
pub struct ReadProgressRequest {
    pub chapter_id: String,
    pub progress: f64,
}
