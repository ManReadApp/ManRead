use std::sync::Arc;

use api_structure::v1::Claim;
use db::{lists::ListDBService, manga::MangaDBService};

use crate::error::{ApiError, ApiResult};

pub struct ListActions {
    pub mangas: Arc<MangaDBService>,
    pub lists: Arc<ListDBService>,
}

impl ListActions {
    fn validate_list_name(name: &str) -> ApiResult<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ApiError::invalid_input("list name cannot be empty"));
        }
        if name.len() > 64 {
            return Err(ApiError::invalid_input("list name too long"));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
        {
            return Err(ApiError::invalid_input(
                "list name contains invalid characters",
            ));
        }
        Ok(())
    }

    pub async fn list(&self, user: &Claim) -> ApiResult<Vec<String>> {
        Ok(self.lists.get(&user.id).await?)
    }
    pub async fn add(&self, name: &str, user: &Claim) -> ApiResult<()> {
        Self::validate_list_name(name)?;
        self.lists.add(&name, &user.id).await?;
        Ok(())
    }
    pub async fn remove(&self, name: &str, user: &Claim) -> ApiResult<()> {
        Self::validate_list_name(name)?;
        self.lists.delete(&name, &user.id).await?;
        Ok(())
    }
    pub async fn add_to_list(&self, list: &str, manga_id: &str, user: &Claim) -> ApiResult<()> {
        Self::validate_list_name(list)?;
        if manga_id.trim().is_empty() {
            return Err(ApiError::invalid_input("manga_id cannot be empty"));
        }
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
        Self::validate_list_name(list)?;
        if manga_id.trim().is_empty() {
            return Err(ApiError::invalid_input("manga_id cannot be empty"));
        }
        self.lists.remove_manga(list, &user.id, &manga_id).await?;
        Ok(())
    }
}
