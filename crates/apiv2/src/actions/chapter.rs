use std::sync::Arc;

use api_structure::v1::{ChapterInfoResponse, EditChapterRequest, Tag};
use chrono::{DateTime, Utc};
use db::{
    chapter::ChapterDBService, manga::MangaDBService, page::PageDBService, tag::TagDBService,
    version::VersionDBService,
};
use storage::{FileId, StorageSystem};

use crate::error::{ApiError, ApiResult};

pub struct ChapterActions {
    pub chapters: Arc<ChapterDBService>,
    pub tags: Arc<TagDBService>,
    pub versions: Arc<VersionDBService>,
    pub mangas: Arc<MangaDBService>,
    pub pages: Arc<PageDBService>,
    pub fs: Arc<StorageSystem>,
}

impl ChapterActions {
    pub async fn add(
        &self,
        manga_id: &str,
        titles: Vec<String>,
        episode: f64,
        version: &str,
        images: Vec<String>,
        tags: Vec<Tag>,
        sources: Vec<String>,
        release_date: Option<DateTime<Utc>>,
    ) -> ApiResult<()> {
        let ch = self.chapters.get(manga_id, episode).await;
        let tags = self.tags.get_ids(tags.into_iter()).await?;
        let version_id = self.versions.get(&version).await?;
        match ch {
            Ok(ch) => {
                if ch.data.versions.contains_key(version) {
                    return Err(ApiError::ChapterVersionAlreadyExists);
                }
                let mut imgs = vec![];
                for image in images {
                    imgs.push(self.fs.take(FileId::new(image)).await?.manga_page(
                        manga_id,
                        ch.id.id().to_string().as_str(),
                        version_id.id().to_string().as_str(),
                    ));
                }
                let images = imgs;
                let imgs = self.pages.add(images).await?;

                self.chapters
                    .add(
                        &ch.id.id().to_string(),
                        titles,
                        tags,
                        sources,
                        version_id,
                        imgs,
                    )
                    .await?;
            }
            Err(_) => {
                self.mangas.exists(&manga_id).await?;

                let chapter_id = self
                    .chapters
                    .create(
                        &manga_id,
                        episode,
                        titles,
                        tags,
                        sources,
                        release_date.map(|v| v.into()),
                    )
                    .await?;
                let mut imgs = vec![];
                for image in images {
                    imgs.push(self.fs.take(FileId::new(image)).await?.manga_page(
                        &manga_id,
                        &chapter_id.id().to_string(),
                        &version_id.id().to_string(),
                    ));
                }
                let images = imgs;
                let imgs = self.pages.add(images).await?;
                self.chapters
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
        Ok(())
    }

    pub async fn delete(&self, chapter_id: &str) -> ApiResult<()> {
        self.chapters.delete(chapter_id).await?;
        Ok(())
    }

    pub async fn edit(&self, data: EditChapterRequest) -> ApiResult<()> {
        let tags = if let Some(tags) = data.tags {
            Some(self.tags.get_ids(tags.items.into_iter()).await?)
        } else {
            None
        };

        let release_date = if data.clear_release_date {
            Some(None)
        } else {
            data.release_date
                .map(|v| {
                    DateTime::from_timestamp_millis(v as i64)
                        .ok_or(ApiError::invalid_input("Invalid release_date timestamp"))
                        .map(|v| Some(v.into()))
                })
                .transpose()?
        };

        let manga_id = self.chapters.get_manga_id(&data.chapter_id).await?;
        self.chapters
            .edit(
                &data.chapter_id,
                data.titles.map(|v| v.items),
                data.chapter,
                tags,
                data.sources.map(|v| v.items),
                release_date,
            )
            .await?;
        self.mangas.regenerate_tags(&manga_id).await?;
        Ok(())
    }

    pub async fn info(&self, chapter_id: &str) -> ApiResult<ChapterInfoResponse> {
        let chapter = self.chapters.get_by_id(&chapter_id).await?;
        let tags = self
            .tags
            .get_tags(chapter.tags.into_iter().map(|v| v.thing.id().to_string()))
            .await?;
        Ok(ChapterInfoResponse {
            titles: chapter.titles,
            chapter: chapter.chapter,
            tags,
            sources: chapter.sources,
            release_date: chapter.release_date.map(|v| v.to_string()),
            versions: chapter
                .versions
                .into_iter()
                .map(|v| (v.0, v.1.thing.id().to_string()))
                .collect(),
        })
    }
}
