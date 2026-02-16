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

impl UserActions {
    pub async fn delete(&self, uid: &str) -> ApiResult<()> {
        self.users.delete(&uid).await?;
        Ok(())
    }

    pub async fn edit(&self, data: UpdateUserRequest, claim: &Claim) -> ApiResult<()> {
        if let Some(name) = data.name {
            self.users.replace_names(&claim.id, name.items).await?;
        }

        if let Some(PasswordChange {
            old_password,
            new_password,
        }) = data.password
        {
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

            self.users
                .replace_icon_ext(&claim.id, temp.ext().unwrap())
                .await?;
            temp.build(&claim.id).await?;
        }

        if let Some(description) = data.description {
            self.users
                .replace_description(&claim.id, description)
                .await?;
        }

        if let Some(links) = data.links {
            self.users.replace_links(&claim.id, links.items).await?;
        }

        if let Some(thumbnail) = data.thumbnail {
            let temp = UserBannerBuilder::from(self.fs.take(FileId::new(thumbnail)).await?);
            self.users
                .replace_thumb_ext(&claim.id, temp.ext().unwrap())
                .await?;
            temp.build(&claim.id).await?;
        }
        Ok(())
    }

    pub async fn info(&self, uid: &str) -> ApiResult<User> {
        let user = self.users.info(&uid).await?;
        Ok(User {
            id: user.id.id().to_string(),
            names: user.data.names,
            role: Role::try_from(user.data.role).unwrap(),
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
            gender: Gender::try_from(user.data.gender as u64).unwrap(),
        })
    }

    pub async fn list(&self, pagination: PaginationRequest) -> ApiResult<Vec<SimpleUser>> {
        let items = self.users.list(pagination.page, pagination.limit).await?;
        Ok(items
            .into_iter()
            .map(|v| SimpleUser {
                id: v.id.id().to_string(),
                names: v.data.names,
                icon_ext: v.data.icon_ext,
                gender: Gender::try_from(v.data.gender as u64).unwrap(),
            })
            .collect::<Vec<_>>())
    }

    pub async fn search(&self, query: SearchRequest) -> ApiResult<Vec<SimpleUser>> {
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
        Ok(found
            .into_iter()
            .map(|v| SimpleUser {
                id: v.id.id().to_string(),
                names: v.data.names,
                icon_ext: v.data.icon_ext,
                gender: Gender::try_from(v.data.gender as u64).unwrap(),
            })
            .collect::<Vec<_>>())
    }
}
