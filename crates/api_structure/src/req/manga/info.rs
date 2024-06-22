use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MangaInfoRequest {
    pub manga_id: String,
}
