use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReaderPageRequest {
    pub chapter_version_id: String,
}
