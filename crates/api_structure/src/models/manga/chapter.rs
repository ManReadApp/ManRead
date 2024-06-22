use serde::{Deserialize, Serialize};

use super::tag::Tag;

#[derive(Serialize, Deserialize)]
pub struct ExternalSite {
    pub url: String,
    pub icon_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<Tag>,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
}
