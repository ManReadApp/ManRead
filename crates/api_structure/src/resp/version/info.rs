use std::collections::HashMap;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct ChapterVersion {
    pub pages: HashMap<u32, Page>,
    pub link: Option<String>,
}

#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct Page {
    pub id: String,
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub ext: String,
}
