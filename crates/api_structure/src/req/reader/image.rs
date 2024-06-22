use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MangaReaderImageRequest {
    pub manga_id: String,
    pub chapter_id: String,
    pub version_id: String,
    pub page: u32,
    pub file_ext: String,
}
