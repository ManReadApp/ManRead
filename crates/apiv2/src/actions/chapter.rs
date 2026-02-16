use std::sync::Arc;

use api_structure::v1::{ChapterInfoResponse, Tag};
use chrono::{DateTime, Utc};
use db::{
    chapter::ChapterDBService, manga::MangaDBService, page::PageDBService, tag::TagDBService,
    version::VersionDBService,
};
use storage::{FileId, StorageSystem};

use crate::error::{ApiError, ApiResult};

pub struct ChapterActions {
    chapters: Arc<ChapterDBService>,
    tags: Arc<TagDBService>,
    versions: Arc<VersionDBService>,
    mangas: Arc<MangaDBService>,
    pages: Arc<PageDBService>,
    fs: StorageSystem,
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

    pub async fn edit() {
        todo!()
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
