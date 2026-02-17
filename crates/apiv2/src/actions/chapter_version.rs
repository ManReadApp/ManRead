use std::sync::Arc;

use api_structure::v1::{PaginationRequest, VersionInfoResponse};
use db::{
    chapter::ChapterDBService,
    page::PageDBService,
    version::{Version, VersionDBService},
    version_link::ChapterVersionDBService,
    RecordIdFunc, SurrealTableInfo,
};
use storage::StorageSystem;

use crate::error::{ApiError, ApiResult};

pub struct ChapterVersionActions {
    pub versions: Arc<VersionDBService>,
    pub chapters: Arc<ChapterDBService>,
    pub chapter_versions: Arc<ChapterVersionDBService>,
    pub pages: Arc<PageDBService>,
    pub fs: Arc<StorageSystem>,
}

impl ChapterVersionActions {
    pub async fn edit(
        &self,
        version_id: &str,
        rename: Option<String>,
        update_translate_opts: Option<String>,
    ) -> ApiResult<()> {
        if version_id.trim().is_empty() {
            return Err(ApiError::invalid_input("version_id cannot be empty"));
        }
        if let Some(name) = rename {
            if name.trim().is_empty() {
                return Err(ApiError::invalid_input("rename cannot be empty"));
            }
            self.versions.rename(&version_id, name).await?;
        }
        if let Some(translate_opts) = update_translate_opts {
            if translate_opts.trim().is_empty() {
                return Err(ApiError::invalid_input(
                    "update_translate_opts cannot be empty",
                ));
            }
            self.versions
                .update_translate_opts(version_id, translate_opts)
                .await?;
        }
        Ok(())
    }

    pub async fn list(&self, pagination: PaginationRequest) -> ApiResult<Vec<VersionInfoResponse>> {
        if pagination.page == 0 {
            return Err(ApiError::invalid_input("page must be >= 1"));
        }
        if pagination.limit == 0 {
            return Err(ApiError::invalid_input("limit must be >= 1"));
        }
        let versions = self
            .versions
            .list(pagination.page, pagination.limit)
            .await?;
        Ok(versions
            .into_iter()
            .map(|v| VersionInfoResponse {
                id: v.id.id().to_string(),
                name: v.data.name,
                translate_opts: v.data.translate_opts,
            })
            .collect())
    }

    pub async fn delete(&self, chapter_id: &str, version_id: &str) -> ApiResult<()> {
        if chapter_id.trim().is_empty() {
            return Err(ApiError::invalid_input("chapter_id cannot be empty"));
        }
        if version_id.trim().is_empty() {
            return Err(ApiError::invalid_input("version_id cannot be empty"));
        }
        let manga_id = self.chapters.get_manga_id(chapter_id).await?;
        let chapter = self.chapters.get_by_id(chapter_id).await?;
        let version_key = RecordIdFunc::from((Version::name(), version_id)).to_string();
        let chapter_version_id = chapter
            .versions
            .get(&version_key)
            .ok_or(ApiError::NotFoundInDB)?
            .id()
            .to_string();
        let chapter_version = self.chapter_versions.get(&chapter_version_id).await?;
        let pages = self.pages.get(chapter_version.pages.clone()).await?;

        self.chapters.delete_version(chapter_id, version_id).await?;
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
        Ok(())
    }
}
