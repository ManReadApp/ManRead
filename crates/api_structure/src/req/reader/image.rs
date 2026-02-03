use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct MangaReaderImageRequest {
    pub manga_id: String,
    pub chapter_id: String,
    pub version_id: String,
    pub page: u32,
    pub file_ext: String,
}

pub type MangaReaderTranslationRequest = MangaReaderImageRequest;
