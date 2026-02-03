use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::tag::Tag;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct ExternalSite {
    pub url: String,
    pub icon_uri: String,
}

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct Chapter {
    pub id: String,
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<Tag>,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
}
