use std::{collections::HashMap, fmt::Display};

use crate::models::manga::{status::Status, tag::Tag};
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct AddMangaRequest {
    pub names: HashMap<String, Vec<String>>,
    pub kind: String,
    pub status: Status,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub image_temp_name: String,
    pub authors: Vec<String>,
    pub publishers: Vec<String>,
    pub artists: Vec<String>,
    pub sources: Vec<String>,
    pub scrapers: Vec<Scrapers>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateMangaRequest {
    pub id: String,
    pub names: HashMap<String, Vec<String>>,
    pub kind: String,
    pub status: u64,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub author_ids: Vec<String>,
    pub artist_ids: Vec<String>,
    pub sources: Vec<String>,
    pub scrapers: Vec<Scrapers>,
}

#[derive(Deserialize, Serialize, Clone, Default, ApiComponent, JsonSchema)]
pub struct Scrapers {
    pub channel: String,
    pub url: String,
}

impl Display for Scrapers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.channel, self.url)
    }
}
