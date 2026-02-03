use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, ApiComponent, JsonSchema)]
pub struct VersionInfoResponse {
    pub id: String,
    pub name: String,
    pub translate_opts: Option<String>,
}
