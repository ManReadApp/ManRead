use std::sync::Arc;

use api_structure::v1::{Claim, PaginationRequest};
use db::{lists::ListDBService, manga::MangaDBService};

use crate::error::ApiResult;

pub struct ListActions {
    mangas: Arc<MangaDBService>,
    lists: Arc<ListDBService>,
}

impl ListActions {
    pub async fn list(&self, user: &Claim) -> ApiResult<Vec<String>> {
        Ok(self.lists.get(&user.id).await?)
    }
    pub async fn add(&self, name: &str, user: &Claim) -> ApiResult<()> {
        self.lists.add(&name, &user.id).await?;
        Ok(())
    }
    pub async fn remove(&self, name: &str, user: &Claim) -> ApiResult<()> {
        self.lists.delete(&name, &user.id).await?;
        Ok(())
    }
    pub async fn add_to_list(&self, list: &str, manga_id: &str, user: &Claim) -> ApiResult<()> {
        self.mangas.exists(manga_id).await?;
        self.lists.add_manga(&list, &user.id, &manga_id).await?;
        Ok(())
    }
    pub async fn remove_from_list(
        &self,
        list: &str,
        manga_id: &str,
        user: &Claim,
    ) -> ApiResult<()> {
        self.lists.remove_manga(list, &user.id, &manga_id).await?;
        Ok(())
    }
}
