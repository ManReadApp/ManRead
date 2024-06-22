use std::collections::HashMap;

use crate::models::manga::tag::Tag;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AddMangaRequest {
    pub names: HashMap<String, Vec<String>>,
    pub kind: String,
    pub tags: Vec<Tag>,
    pub image_temp_name: String,
    pub scape: Option<String>,
    pub sources: Vec<String>,
}
