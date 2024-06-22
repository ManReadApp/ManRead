use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReaderChapter {
    pub chapter_id: String,
    pub titles: Vec<String>,
    pub chapter: f64,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
    ///Version, versionchapter
    pub versions: HashMap<String, String>,
}
