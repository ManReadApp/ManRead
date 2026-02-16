use std::sync::Arc;

use api_structure::v1::{PaginationRequest, VersionInfoResponse};
use db::{chapter::ChapterDBService, version::VersionDBService};

use crate::error::ApiResult;

pub struct ChapterVersionActions {
    pub versions: Arc<VersionDBService>,
    pub chapters: Arc<ChapterDBService>,
}

impl ChapterVersionActions {
    pub async fn edit(
        &self,
        version_id: &str,
        rename: Option<String>,
        update_translate_opts: Option<String>,
    ) -> ApiResult<()> {
        if let Some(name) = rename {
            self.versions.rename(&version_id, name).await?;
        }
        if let Some(translate_opts) = update_translate_opts {
            self.versions
                .update_translate_opts(version_id, translate_opts)
                .await?;
        }
        Ok(())
    }

    pub async fn list(&self, pagination: PaginationRequest) -> ApiResult<Vec<VersionInfoResponse>> {
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
        self.chapters
            .delete_version(&chapter_id, &version_id)
            .await?;
        Ok(())
    }
}
