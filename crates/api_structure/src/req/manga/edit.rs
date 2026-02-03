use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::manga::{status::Status, tag::Tag};

use super::add::Scrapers;

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct EditMangaRequest {
    pub manga_id: String,
    pub names: HashMap<String, Vec<String>>,
    pub kind: String,
    pub status: Status,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub authors: Vec<String>,
    pub publishers: Vec<String>,
    pub artists: Vec<String>,
    pub sources: Vec<String>,
    pub scrapers: Vec<Scrapers>,
}
