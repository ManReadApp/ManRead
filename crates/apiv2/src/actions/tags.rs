use std::sync::Arc;

use api_structure::v1::Tag;
use db::tag::TagDBService;

use crate::error::ApiResult;

pub struct TagActions {
    tags: Arc<TagDBService>,
}

impl TagActions {
    pub async fn search(&self, query: &str) -> ApiResult<Vec<Tag>> {
        Ok(self.tags.search(query).await?)
    }
}
