use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(ApiComponent, Deserialize, Serialize, JsonSchema)]
pub struct ToScrape {
    pub manga_id: String,
    pub names: HashMap<String, Vec<String>>,
    pub version: String,
    pub version_id: String,
}

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct ScrapeChapterListRequest {
    pub manga_id: String,
    pub version_id: String,
}

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct ScrapeChapterListResponse {
    pub id: String,
    pub chapter: f64,
    pub name: Vec<String>,
    pub link: Option<String>,
    pub state: String,
}
