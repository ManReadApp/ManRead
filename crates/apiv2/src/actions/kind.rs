use std::sync::Arc;

use db::kind::KindDBService;

use crate::error::ApiResult;

pub struct KindActions {
    pub kinds: Arc<KindDBService>,
}

impl KindActions {
    pub async fn list(&self) -> ApiResult<Vec<String>> {
        Ok(self.kinds.all().await?)
    }
}
