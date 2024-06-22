use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MangaCoverRequest {
    pub manga_id: String,
    pub file_ext: String,
}
