use crate::env::config::Config;
use crate::errors::{ApiError, ApiResult};
use crate::services::db::chapter::ChapterDBService;
use crate::services::db::chapter_version::ChapterVersionDBService;
use crate::services::db::manga::MangaDBService;
use crate::services::db::manga_kind::MangaKindDBService;
use crate::services::db::page::PageDBService;
use crate::services::db::progress::ProgressDBService;
use actix_files::NamedFile;
use actix_web::post;
use std::sync::Arc;
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::protect;
use api_structure::auth::jwt::Claim;
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::image::MangaReaderImageRequest;
use api_structure::reader::{
    MangaReaderRequest, MangaReaderResponse, Progress, ReaderPage, ReaderPageRequest,
    ReaderPageResponse, TranslationArea,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::read_to_string;

#[post("/pages")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn get_pages(
    Json(req): Json<ReaderPageRequest>,
    cvs: Data<ChapterVersionDBService>,
    page_s: Data<PageDBService>,
) -> ApiResult<Json<ReaderPageResponse>> {
    let mut pages = Vec::new();
    for page in cvs.get(&req.chapter_version_id).await? {
        let page_id = page.thing.id().to_string();
        let page = page_s.get(page).await?;
        pages.push((
            page.page,
            ReaderPage {
                page_id,
                width: page.width,
                height: page.height,
                ext: page.ext,
                translation: page.translation,
                progress: Progress {
                    width_start: 0.0,
                    height_start: 0.0,
                    width_end: 0.0,
                    height_end: 0.0,
                },
            },
        ));
    }
    let mut max_width = 0;
    let mut max_height = 0;
    for (_, page) in pages.iter() {
        max_width += page.width;
        max_height += page.height;
    }
    pages.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut width = 0;
    let mut height = 0;
    let max_width = max_width as f64;
    let max_height = max_height as f64;
    for (_, page) in &mut pages {
        let width_start = width as f64 / max_width;
        let height_start = height as f64 / max_height;
        width += page.width;
        height += page.height;
        let width_end = width as f64 / max_width;
        let height_end = height as f64 / max_height;
        page.progress = Progress {
            width_start,
            height_start,
            width_end,
            height_end,
        }
    }
    Ok(Json(ReaderPageResponse {
        version_id: req.chapter_version_id,
        hide_top: 0.0,
        hide_bottom: 0.0,
        pages: pages.into_iter().map(|(a, b)|(a, Arc::new(b))).collect(),
    }))
}

#[post("/reader_info")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn info(
    Json(req): Json<MangaReaderRequest>,
    manga: Data<MangaDBService>,
    chapter_s: Data<ChapterDBService>,
    progress_s: Data<ProgressDBService>,
    user: ReqData<Claim>,
    kind_s: Data<MangaKindDBService>,
) -> ApiResult<Json<MangaReaderResponse>> {
    let manga = manga.get(req.manga_id.as_str()).await?;
    let kind = kind_s
        .get_kind(&manga.data.kind.thing.id().to_string())
        .await
        .ok_or(ApiError::db_error())?;
    let mut chapters = vec![];
    for chapter in manga.data.chapters {
        chapters.push(chapter_s.get_reader(chapter).await?);
    }
    chapters.sort_by(|a, b| a.chapter.partial_cmp(&b.chapter).unwrap());
    let (open_chapter, progress) = match req.chapter_id {
        None => progress_s
            .get_progress(user.id.as_str(), manga.id)
            .await
            .unwrap_or_else(|| {
                (
                    chapters
                        .first()
                        .map(|v| v.chapter_id.clone())
                        .unwrap_or_default(),
                    0.0,
                )
            }),
        Some(v) => (v, 0.0),
    };
    Ok(Json(MangaReaderResponse {
        manga_id: req.manga_id,
        titles: manga.data.titles,
        kind: kind.kind,
        description: manga.data.description,
        chapters,
        favorite: false,
        open_chapter,
        progress,
    }))
}

#[post("/chapter_page")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn chapter_page_route(
    Json(data): Json<MangaReaderImageRequest>,
    config: Data<Config>,
) -> ApiResult<NamedFile> {
    if let Some(version_id) = data.version_id.strip_prefix("chapter_versions:") {
        Ok(NamedFile::open(
            config
                .root_folder
                .join("mangas")
                .join(data.manga_id)
                .join(data.chapter_id)
                .join(version_id)
                .join(format!("{}.{}", data.page, data.file_ext)),
        )?)
    } else {
        Err(ApiErr {
            message: Some("invalid version_id_prefix".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into())
    }
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

#[post("page_translation")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
async fn translation(
    Json(data): Json<MangaReaderImageRequest>,
    config: Data<Config>,
) -> ApiResult<Json<Vec<TranslationArea>>> {
    if let Some(version_id) = data.version_id.strip_prefix("chapter_versions:") {
        let s = read_to_string(File::open(
            config
                .root_folder
                .join("mangas")
                .join(data.manga_id)
                .join(data.chapter_id)
                .join(version_id)
                .join(format!("{}.json", data.page)),
        )?)?;
        let mut v: TranslationResponse = serde_json::from_str(&s)?;
        Ok(Json(
            v.images.remove(0).into_iter().map(|v| v.into()).collect(),
        ))
    } else {
        Err(ApiErr {
            message: Some("invalid version_id_prefix".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into())
    }
}

#[derive(Serialize, Deserialize)]
struct Translation {
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
struct TranslationResponse {
    pub images: Vec<Vec<Translation>>,
}

pub fn is_valid_translation(s: &str) -> bool{
    let v: Result<TranslationResponse, _> = serde_json::from_str(s);
    v.is_ok()
}
