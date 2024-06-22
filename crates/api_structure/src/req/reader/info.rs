use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MangaReaderRequest {
    pub manga_id: String,
    pub chapter_id: Option<String>,
}
