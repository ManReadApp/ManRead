use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::models::{auth::role::Permission, manga::tag::Tag};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        chapter::ChapterDBService, manga::MangaDBService, page::PageDBService, tag::TagDBService,
        version::VersionDBService,
    },
    services::file::FileService,
    Config,
};

#[derive(serde::Deserialize, Debug, JsonSchema, ApiComponent)]
pub struct NewChapterRequest {
    manga_id: String,
    tags: Vec<Tag>,
    titles: Vec<String>,
    sources: Vec<String>,
    version: String,
    episode: f64,
    images: Vec<String>,
    release_date: Option<DateTime<Utc>>,
}
#[api_operation(
    tag = "chapter",
    summary = "Adds a chapter to an existing chapter",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(chapter): Json<NewChapterRequest>,
    manga_service: Data<MangaDBService>,
    chapter_service: Data<ChapterDBService>,
    page_service: Data<PageDBService>,
    version_service: Data<VersionDBService>,
    file_service: Data<FileService>,
    tag_service: Data<TagDBService>,
    config: Data<Config>,
) -> ApiResult<CreatedJson<u8>> {
    let ch = chapter_service
        .get(&chapter.manga_id, chapter.episode)
        .await;
    let tags = tag_service.get_ids(chapter.tags.into_iter()).await?;
    let version_id = version_service.get(&chapter.version).await?;
    match ch {
        Ok(ch) => {
            if ch.data.versions.contains_key(&chapter.version) {
                return Err(ApiError::ChapterVersionAlreadyExists);
            }
            let images = chapter
                .images
                .into_iter()
                .map(|v| file_service.take(&v))
                .collect::<ApiResult<Vec<_>>>()?;
            let imgs = page_service
                .add(
                    &config.root_folder,
                    &chapter.manga_id,
                    ch.id.id().to_string().as_str(),
                    version_id.id().to_string().as_str(),
                    images,
                )
                .await?;

            chapter_service
                .add(
                    &ch.id.id().to_string(),
                    chapter.titles,
                    tags,
                    chapter.sources,
                    version_id,
                    imgs,
                )
                .await?;
        }
        Err(_) => {
            manga_service.exists(&chapter.manga_id).await?;
            let images = chapter
                .images
                .into_iter()
                .map(|v| file_service.take(&v))
                .collect::<ApiResult<Vec<_>>>()?;

            let chapter_id = chapter_service
                .create(
                    &chapter.manga_id,
                    chapter.episode,
                    chapter.titles,
                    tags,
                    chapter.sources,
                    chapter.release_date.map(|v| v.into()),
                )
                .await?;
            let imgs = page_service
                .add(
                    &config.root_folder,
                    &chapter.manga_id,
                    &chapter_id.id().to_string(),
                    &version_id.id().to_string(),
                    images,
                )
                .await?;
            chapter_service
                .add(
                    &chapter_id.id().to_string(),
                    vec![],
                    vec![],
                    vec![],
                    version_id,
                    imgs,
                )
                .await?;
        }
    }
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/add").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
