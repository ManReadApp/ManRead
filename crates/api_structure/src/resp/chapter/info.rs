use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;

use crate::models::manga::tag::Tag;

#[derive(Serialize, ApiComponent, JsonSchema)]
pub struct ChapterInfoResponse {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<Tag>,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
    pub versions: HashMap<String, String>,
}
