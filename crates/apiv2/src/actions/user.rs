use std::sync::Arc;

use api_structure::v1::{
    Claim, Gender, PaginationRequest, PasswordChange, Role, SearchRequest, SimpleUser,
    UpdateUserRequest, User,
};
use db::{tag::TagDBService, user::UserDBService};
use storage::{FileBuilderExt as _, FileId, StorageSystem, UserBannerBuilder};

use crate::{
    actions::crytpo::CryptoService,
    error::{ApiError, ApiResult},
};

pub struct UserActions {
    pub users: Arc<UserDBService>,
    pub crypto: Arc<CryptoService>,
    pub fs: Arc<StorageSystem>,
    pub tags: Arc<TagDBService>,
}

fn reorder(names: &mut Vec<String>, query: &str) {
    names.sort_by_key(|name| {
        if name.starts_with(query) {
            0
        } else if name.contains(query) {
            1
        } else {
            2
        }
    });
}

fn role_from_db(value: u32) -> ApiResult<Role> {
    Role::try_from(value).map_err(|_| ApiError::write_error("invalid role value in database"))
}

fn gender_from_db(value: u64) -> ApiResult<Gender> {
    Gender::try_from(value).map_err(|_| ApiError::write_error("invalid gender value in database"))
}

impl UserActions {
    pub async fn delete(&self, uid: &str) -> ApiResult<()> {
        if uid.trim().is_empty() {
            return Err(ApiError::invalid_input("uid cannot be empty"));
        }
        self.users.delete(&uid).await?;
        Ok(())
    }

    pub async fn edit(&self, data: UpdateUserRequest, claim: &Claim) -> ApiResult<()> {
        if let Some(name) = data.name {
            if name.items.is_empty() || name.items.iter().any(|v| v.trim().is_empty()) {
                return Err(ApiError::invalid_input("name list cannot be empty"));
            }
            self.users.replace_names(&claim.id, name.items).await?;
        }

        if let Some(PasswordChange {
            old_password,
            new_password,
        }) = data.password
        {
            if new_password.trim().is_empty() {
                return Err(ApiError::invalid_input("new_password cannot be empty"));
            }
            let user = self.users.get_by_id(&claim.id).await?;
            let verify = self
                .crypto
                .verify_hash(old_password, user.data.password)
                .await;
            if verify {
                self.users
                    .set_password(&claim.id, self.crypto.hash_password(&new_password).await?)
                    .await?;
            } else {
                return Err(ApiError::PasswordIncorrect);
            }
        }

        if let Some(icon_temp_name) = data.icon_temp_name {
            let temp = self
                .fs
                .get_user_cover(Some(FileId::new(icon_temp_name)))
                .await?;

            self.users.replace_icon_ext(&claim.id, temp.ext()?).await?;
            temp.build(&claim.id).await?;
        }

        if let Some(description) = data.description {
            self.users
                .replace_description(&claim.id, description)
                .await?;
        }

        if let Some(links) = data.links {
            if links.items.iter().any(|v| v.trim().is_empty()) {
                return Err(ApiError::invalid_input("links cannot contain empty items"));
            }
            self.users.replace_links(&claim.id, links.items).await?;
        }

        if let Some(thumbnail) = data.thumbnail {
            let temp = UserBannerBuilder::from(self.fs.take(FileId::new(thumbnail)).await?);
            self.users.replace_thumb_ext(&claim.id, temp.ext()?).await?;
            temp.build(&claim.id).await?;
        }
        Ok(())
    }

    pub async fn info(&self, uid: &str) -> ApiResult<User> {
        if uid.trim().is_empty() {
            return Err(ApiError::invalid_input("uid cannot be empty"));
        }
        let user = self.users.info(&uid).await?;
        Ok(User {
            id: user.id.id().to_string(),
            names: user.data.names,
            role: role_from_db(user.data.role)?,
            tags: self
                .tags
                .get_tags(user.data.tags.into_iter().map(|v| v.thing.id().to_string()))
                .await?,
            achievements: user.data.achievements.into_iter().map(u64::from).collect(),
            bio: user.data.bio,
            location: user.data.location,
            links: user.data.links,
            thumb_ext: user.data.thumb_ext,
            icon_ext: user.data.icon_ext,
            gender: gender_from_db(user.data.gender as u64)?,
        })
    }

    pub async fn list(&self, pagination: PaginationRequest) -> ApiResult<Vec<SimpleUser>> {
        if pagination.page == 0 {
            return Err(ApiError::invalid_input("page must be >= 1"));
        }
        if pagination.limit == 0 {
            return Err(ApiError::invalid_input("limit must be >= 1"));
        }
        let items = self.users.list(pagination.page, pagination.limit).await?;
        items
            .into_iter()
            .map(|v| {
                Ok(SimpleUser {
                    id: v.id.id().to_string(),
                    names: v.data.names,
                    icon_ext: v.data.icon_ext,
                    gender: gender_from_db(v.data.gender as u64)?,
                })
            })
            .collect()
    }

    pub async fn search(&self, query: SearchRequest) -> ApiResult<Vec<SimpleUser>> {
        if query.page == 0 {
            return Err(ApiError::invalid_input("page must be >= 1"));
        }
        if query.limit == 0 {
            return Err(ApiError::invalid_input("limit must be >= 1"));
        }
        if query.query.trim().is_empty() {
            return Err(ApiError::invalid_input("query cannot be empty"));
        }
        let items = self
            .users
            .search(&query.query, query.page, query.limit)
            .await?;
        let (mut found, not_found): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|mut v| {
                reorder(&mut v.data.names, &query.query);
                v
            })
            .partition(|v| {
                v.data
                    .names
                    .get(0)
                    .map(|v| v.to_lowercase().starts_with(&query.query.to_lowercase()))
                    .unwrap_or(false)
            });
        found.extend(not_found);
        found
            .into_iter()
            .map(|v| {
                Ok(SimpleUser {
                    id: v.id.id().to_string(),
                    names: v.data.names,
                    icon_ext: v.data.icon_ext,
                    gender: gender_from_db(v.data.gender as u64)?,
                })
            })
            .collect()
    }
}
