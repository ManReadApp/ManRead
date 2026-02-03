use std::{collections::HashMap, fs::read_to_string};

use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{auth::role::Permission, reader::translation::TranslationArea},
    req::reader::image::{MangaReaderImageRequest, MangaReaderTranslationRequest},
};
use apistos::{actix::CreatedJson, api_operation};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, ApiResult},
    Config,
};

#[api_operation(
    tag = "image",
    summary = "Gets a specific translation of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaReaderImageRequest>,
    config: Data<Config>,
) -> ApiResult<CreatedJson<Vec<TranslationArea>>> {
    get_translation(&config, &data).map(CreatedJson)
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/translation").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}

pub fn get_translation(
    config: &Config,
    data: &MangaReaderTranslationRequest,
) -> ApiResult<Vec<TranslationArea>> {
    if data.manga_id.contains("/") || data.chapter_id.contains("/") || data.version_id.contains("/")
    {
        return Err(ApiError::InvalidImageId);
    }
    let path = config
        .root_folder
        .join("mangas")
        .join(&data.manga_id)
        .join(&data.chapter_id)
        .join(&data.version_id)
        .join(format!("{}.json", data.page));
    if path.is_file() {
        //TODO: rewrite parsing
        let mut v: TranslationResponse = serde_json::from_str(&read_to_string(path)?)?;
        Ok(v.images.remove(0).into_iter().map(|v| v.into()).collect())
    } else {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize)]
pub struct Translation {
    #[serde(rename = "translatedText")]
    pub translated_text: String,
    #[serde(rename = "minX")]
    pub min_x: u32,
    #[serde(rename = "minY")]
    pub min_y: u32,
    #[serde(rename = "maxX")]
    pub max_x: u32,
    #[serde(rename = "maxY")]
    pub max_y: u32,
    pub background: String,
}

#[derive(Serialize, Deserialize)]
pub struct TranslationResponse {
    pub images: Vec<Vec<Translation>>,
}

impl From<Translation> for TranslationArea {
    fn from(value: Translation) -> Self {
        let mut hm = HashMap::new();
        hm.insert("eng_ichigo".to_string(), value.translated_text);
        Self {
            translated_text: hm,
            min_x: value.min_x,
            min_y: value.min_y,
            max_x: value.max_x,
            max_y: value.max_y,
            text_color: [0; 3],
            outline_color: [255; 3],
            background: value.background,
        }
    }
}
