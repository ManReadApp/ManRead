use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    info::{Tag, TagSex},
    RequestImpl,
};

#[derive(Deserialize, Serialize)]
pub struct AddMangaRequest {
    pub names: HashMap<String, Vec<String>>,
    pub kind: String,
    pub tags: Vec<Tag>,
    pub image_temp_name: String,
    pub scape: Option<String>,
    pub sources: Vec<String>,
}

impl RequestImpl for AddMangaRequest {
    const ROUTE: &'static str = "manga/add";

    const AUTH: bool = true;
}

pub type Kinds = Vec<String>;
impl RequestImpl for Kinds {
    const ROUTE: &'static str = "manga/kinds";

    const AUTH: bool = true;
}

#[derive(Deserialize, Serialize)]
pub struct TagsRequest {
    pub query: String,
    pub limit: usize,
    pub sex: TagSex,
}

impl RequestImpl for TagsRequest {
    const ROUTE: &'static str = "manga/tags";

    const AUTH: bool = true;
}
