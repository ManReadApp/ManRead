use std::sync::Arc;

use api_structure::v1::{ChapterInfoResponse, EditChapterRequest, Tag};
use chrono::{DateTime, Utc};
use db::{
    chapter::ChapterDBService, manga::MangaDBService, page::PageDBService, tag::TagDBService,
    version::VersionDBService, version_link::ChapterVersionDBService,
};
use storage::{FileId, StorageSystem};

use crate::error::{ApiError, ApiResult};

pub struct ChapterActions {
    pub chapters: Arc<ChapterDBService>,
    pub tags: Arc<TagDBService>,
    pub versions: Arc<VersionDBService>,
    pub chapter_versions: Arc<ChapterVersionDBService>,
    pub mangas: Arc<MangaDBService>,
    pub pages: Arc<PageDBService>,
    pub fs: Arc<StorageSystem>,
}

impl ChapterActions {
    fn validate_non_empty_items(field: &str, items: &[String]) -> ApiResult<()> {
        if items.iter().any(|value| value.trim().is_empty()) {
            return Err(ApiError::invalid_input(&format!(
                "{field} cannot contain empty values"
            )));
        }
        Ok(())
    }

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
        if manga_id.trim().is_empty() {
            return Err(ApiError::invalid_input("manga_id cannot be empty"));
        }
        if version.trim().is_empty() {
            return Err(ApiError::invalid_input("version cannot be empty"));
        }
        if !episode.is_finite() {
            return Err(ApiError::invalid_input("episode must be finite"));
        }
        if episode < 0.0 {
            return Err(ApiError::invalid_input("episode must be >= 0"));
        }
        if images.is_empty() {
            return Err(ApiError::invalid_input("images cannot be empty"));
        }
        if titles.is_empty() {
            return Err(ApiError::invalid_input("titles cannot be empty"));
        }
        Self::validate_non_empty_items("titles", &titles)?;
        Self::validate_non_empty_items("images", &images)?;
        Self::validate_non_empty_items("sources", &sources)?;

        let ch = self.chapters.get(manga_id, episode).await;
        let tags = self.tags.get_ids(tags.into_iter()).await?;
        let version_id = self.versions.get(&version).await?;
        let version_key = version_id.to_string();
        match ch {
            Ok(ch) => {
                if ch.data.versions.contains_key(&version_key) {
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
                if let Err(err) = self
                    .chapters
                    .add(
                        &ch.id.id().to_string(),
                        titles,
                        tags,
                        sources,
                        version_id,
                        imgs.clone(),
                    )
                    .await
                {
                    let _ = self.pages.delete(imgs).await;
                    return Err(err.into());
                }
            }
            Err(_) => {
                self.mangas.exists(&manga_id).await?;
                let mut pending_files = Vec::new();
                for image in images {
                    pending_files.push(self.fs.take(FileId::new(image)).await?);
                }

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
                for image in pending_files {
                    imgs.push(image.manga_page(
                        &manga_id,
                        &chapter_id.id().to_string(),
                        &version_id.id().to_string(),
                    ));
                }
                let images = imgs;
                let chapter_id_str = chapter_id.id().to_string();
                let imgs = match self.pages.add(images).await {
                    Ok(images) => images,
                    Err(err) => {
                        let _ = self.chapters.delete(&chapter_id_str).await;
                        return Err(err.into());
                    }
                };
                if let Err(err) = self
                    .chapters
                    .add(
                        &chapter_id_str,
                        vec![],
                        vec![],
                        vec![],
                        version_id,
                        imgs.clone(),
                    )
                    .await
                {
                    let _ = self.pages.delete(imgs).await;
                    let _ = self.chapters.delete(&chapter_id_str).await;
                    return Err(err.into());
                }
            }
        }
        Ok(())
    }

    pub async fn delete(&self, chapter_id: &str) -> ApiResult<()> {
        if chapter_id.trim().is_empty() {
            return Err(ApiError::invalid_input("chapter_id cannot be empty"));
        }
        let chapter = self.chapters.get_by_id(chapter_id).await?;
        let manga_id = self.chapters.get_manga_id(chapter_id).await?;
        for (version_key, connection) in &chapter.versions {
            let version_id = version_key
                .split_once(':')
                .map(|(_, id)| id)
                .unwrap_or(version_key.as_str());
            let chapter_version = self
                .chapter_versions
                .get(&connection.id().to_string())
                .await?;
            let pages = self.pages.get(chapter_version.pages.clone()).await?;
            self.pages.delete(chapter_version.pages).await?;

            for page in pages {
                let key = format!(
                    "mangas/{}/{}/{}/{}.{}",
                    manga_id,
                    chapter_id,
                    version_id,
                    page.data.page,
                    page.data.ext.trim_start_matches('.'),
                );
                if let Err(err) = self.fs.delete_key(&key).await {
                    if !matches!(err, storage::StorageError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound)
                    {
                        return Err(err.into());
                    }
                }
            }
        }
        self.chapters.delete(chapter_id).await?;
        Ok(())
    }

    pub async fn edit(&self, data: EditChapterRequest) -> ApiResult<()> {
        if data.chapter_id.trim().is_empty() {
            return Err(ApiError::invalid_input("chapter_id cannot be empty"));
        }
        if let Some(chapter) = data.chapter {
            if !chapter.is_finite() {
                return Err(ApiError::invalid_input("chapter must be finite"));
            }
            if chapter < 0.0 {
                return Err(ApiError::invalid_input("chapter must be >= 0"));
            }
        }
        if data
            .titles
            .as_ref()
            .map(|v| v.items.is_empty() || v.items.iter().any(|s| s.trim().is_empty()))
            .unwrap_or(false)
        {
            return Err(ApiError::invalid_input(
                "titles cannot contain empty values",
            ));
        }
        if data
            .sources
            .as_ref()
            .map(|v| v.items.is_empty() || v.items.iter().any(|s| s.trim().is_empty()))
            .unwrap_or(false)
        {
            return Err(ApiError::invalid_input(
                "sources cannot contain empty values",
            ));
        }
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
                    i64::try_from(v)
                        .ok()
                        .and_then(DateTime::from_timestamp_millis)
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
        if chapter_id.trim().is_empty() {
            return Err(ApiError::invalid_input("chapter_id cannot be empty"));
        }
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
