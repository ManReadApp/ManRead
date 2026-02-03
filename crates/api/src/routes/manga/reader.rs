use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{jwt::Claim, role::Permission},
        reader::chapter::ReaderChapter,
    },
    resp::reader::MangaReaderResponse,
};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        chapter::ChapterDBService, kind::KindDBService, lists::ListDBService,
        manga::MangaDBService, progress::UserProgressDBService,
    },
};

#[derive(serde::Deserialize, ApiComponent, JsonSchema)]
struct ReaderRequest {
    manga_id: String,
    chapter_id: Option<String>,
}

#[api_operation(
    tag = "manga",
    summary = "Gets all the info for the manga home",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<ReaderRequest>,
    manga_service: Data<MangaDBService>,
    progress_service: Data<UserProgressDBService>,
    chapter_service: Data<ChapterDBService>,
    list_service: Data<ListDBService>,
    kind_service: Data<KindDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<MangaReaderResponse>> {
    let manga = manga_service.get(&data.manga_id).await?;
    let (chapter, progress) = match data.chapter_id {
        Some(v) => (v, 0.0_f64),
        None => match progress_service
            .get_progress(&user.id, &data.manga_id)
            .await
            .map(|v| (v.0.id().to_string(), v.1))
        {
            Ok(v) => v,
            Err(_) => (
                manga
                    .chapters
                    .get(0)
                    .ok_or(ApiError::NotFoundInDB)?
                    .id()
                    .to_string(),
                0.0_f64,
            ),
        },
    };
    let mut chapters: Vec<ReaderChapter> = chapter_service
        .get__detail(manga.chapters.into_iter())
        .await?
        .into_iter()
        .map(|v| ReaderChapter {
            chapter_id: v.id.id().to_string(),
            titles: v.data.titles,
            chapter: v.data.chapter,
            sources: v.data.sources,
            release_date: v.data.release_date.map(|v| v.into_inner().0),
            versions: v
                .data
                .versions
                .into_iter()
                .map(|v| (v.0, v.1.id().to_string()))
                .collect(),
        })
        .collect();
    chapters.sort_by(|a, b| {
        a.chapter
            .partial_cmp(&b.chapter)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(CreatedJson(MangaReaderResponse {
        favorite: list_service.is_favorite(&data.manga_id, &user.id).await,
        manga_id: data.manga_id,
        titles: manga.titles,
        kind: kind_service.get_name(manga.kind).await?,
        description: manga.description,
        chapters,
        open_chapter: chapter,
        progress,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/reader").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
