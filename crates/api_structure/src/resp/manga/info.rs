use crate::{
    models::manga::{
        chapter::{Chapter, ExternalSite},
        status::Status,
        tag::Tag,
        visiblity::Visibility,
    },
    req::manga::add::Scrapers,
};
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct MangaInfoResponse {
    pub manga_id: String,
    pub titles: HashMap<String, Vec<String>>,
    pub kind: String,
    pub description: Option<String>,
    pub tags: Vec<Tag>,
    pub status: Status,
    pub visibility: Visibility,
    pub uploader: String,
    pub my: bool,
    pub artists: Vec<String>,
    pub authors: Vec<String>,
    pub publishers: Vec<String>,
    pub cover_ext: Vec<Option<String>>,
    pub chapters: Vec<Chapter>,
    pub sources: Vec<ExternalSite>,
    pub scrapers: Vec<Scrapers>,
    pub relations: Vec<(String, String)>,
    pub scraper: bool,
    pub favorite: bool,
    pub progress: bool,
}
