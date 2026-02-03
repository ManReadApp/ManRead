use std::collections::HashMap;

use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct ReaderChapter {
    pub chapter_id: String,
    pub titles: Vec<String>,
    pub chapter: f64,
    pub sources: Vec<String>,
    pub release_date: Option<DateTime<Utc>>,
    ///Version, versionchapter
    pub versions: HashMap<String, String>,
}
