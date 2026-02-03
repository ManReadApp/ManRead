use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, JsonSchema, ApiComponent)]
pub struct ChapterVersionDeleteRequest {
    pub chapter_id: String,
    pub version_id: String,
}
