use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, ApiComponent, JsonSchema)]
pub struct ChapterVersionEditRequest {
    pub id: String,
    pub rename: Option<String>,
    pub update_translate_opts: Option<String>,
}
