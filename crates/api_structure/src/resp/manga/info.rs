use crate::models::manga::{
    chapter::{Chapter, ExternalSite},
    status::Status,
    tag::Tag,
    visiblity::Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
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
    pub cover: u32,
    pub cover_ext: String,
    pub chapters: Vec<Chapter>,
    pub sources: Vec<ExternalSite>,
    pub relations: Vec<(String, String)>,
    pub scraper: bool,
    pub favorite: bool,
    /// manga_id
    pub progress: Option<String>,
}
