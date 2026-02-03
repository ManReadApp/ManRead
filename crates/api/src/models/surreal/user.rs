use std::time::Duration;

use api_structure::models::{auth::role::Role, manga::tag::Tag};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo as _,
    ThingArray,
};

use super::manga::vec_default;
use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::tag::Empty,
    random_string,
    services::achievement::Achievement,
};
pub struct UserDBService {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum NotificationMode {
    Email,
    Push,
}

fn notification_mode_default() -> NotificationMode {
    NotificationMode::Push
}

#[derive(SurrealSelect, Deserialize)]
pub struct SimpleUser {
    pub names: Vec<String>,
    pub icon_ext: Option<String>,
    pub gender: u32,
}

#[derive(SurrealSelect, Deserialize)]
pub struct UserName {
    pub names: Vec<String>,
}

#[derive(SurrealSelect, Deserialize)]
pub struct RoleExpiry {
    pub role: u32,
    pub generated: Option<Datetime>,
}
//TODO: comments/reviews(manga), chat
#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("users")]
#[sql(["DEFINE EVENT user_updated ON TABLE users WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct User {
    /// Usernames
    pub names: Vec<String>,
    /// Email address
    pub email: String,
    /// Password hash
    pub password: String,
    /// Server role
    pub role: u32,
    /// tags
    #[serde(default = "vec_default")]
    pub tags: Vec<RecordIdType<Tag>>,
    /// Achievements
    #[serde(default = "vec_default")]
    pub achievements: Vec<Achievement>,
    /// How the user wants to be notified
    #[serde(default = "notification_mode_default")]
    pub notification_mode: NotificationMode,
    /// More information about the user
    pub bio: Option<String>,
    /// More information about the user(location)
    pub location: Option<String>,
    /// Links to the users other sites
    pub links: Vec<String>,
    /// extension of the thumb
    pub thumb_ext: Option<String>,
    /// extension of the icon
    pub icon_ext: Option<String>,
    /// Birthdate as timestamp
    pub birthdate: Datetime,
    /// Gender as a number
    pub gender: u32,
    /// When the last valid jwt was generated
    pub generated: Option<Datetime>,
    #[opt(exclude = true)]
    /// When the user was updated in the database
    pub updated: Datetime,
    #[opt(exclude = true)]
    /// When the user was created in the database
    pub created: Datetime,
}

#[derive(Deserialize, SurrealSelect)]
pub struct UserRolePassword {
    pub role: u32,
    pub email: String,
    pub password: String,
}

impl UserDBService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_or_create(&self, name: &str) -> ApiResult<RecordIdFunc> {
        Ok(match self.get_by_name(name).await {
            Ok(v) => v.id,
            Err(_) => {
                self.new_user(
                    name.to_owned(),
                    format!("{name}@example.com"),
                    random_string(10),
                    ".tmp",
                    Default::default(),
                    0,
                )
                .await?
                .id
            }
        })
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
        limit: u32,
    ) -> ApiResult<Vec<RecordData<SimpleUser>>> {
        let user: Vec<RecordData<SimpleUser>> = User::search(
            &*DB,
            Some(format!(
                "WHERE names.any(|$s| string::contains(string::lowercase($s), \"{}\")) LIMIT {} START {}",
                query.to_lowercase(),
                limit,
                (page - 1) * limit
            )),
        )
        .await?;
        Ok(user)
    }

    pub async fn new_user(
        &self,
        name: String,
        email: String,
        password: String,
        icon_ext: &str,
        birthdate: DateTime<Utc>,
        gender: u32,
    ) -> ApiResult<RecordData<UserRolePassword>> {
        let user = User {
            names: vec![name],
            email: email.to_lowercase(),
            password,
            role: Role::NotVerified as u32,
            tags: vec![],
            achievements: vec![],
            notification_mode: NotificationMode::Push,
            bio: None,
            location: None,
            links: vec![],
            thumb_ext: None,
            icon_ext: Some(icon_ext.to_owned()),
            birthdate: birthdate.into(),
            gender,
            generated: None,
            updated: Default::default(),
            created: Default::default(),
        }
        .add(&*DB)
        .await?
        .ok_or(ApiError::NotFoundInDB)?;
        Ok(RecordData {
            id: user.id,
            data: UserRolePassword {
                role: user.data.role,
                email: user.data.email.to_lowercase(),
                password: user.data.password,
            },
        })
    }

    pub async fn replace_names(&self, id: &str, names: Vec<String>) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/names", names))
            .await?;
        Ok(())
    }

    pub async fn set_password(&self, id: &str, password: String) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/password", password))
            .await?;
        Ok(())
    }

    pub async fn list(&self, page: u32, limit: u32) -> ApiResult<Vec<RecordData<SimpleUser>>> {
        Ok(User::search(
            &*DB,
            Some(format!("LIMIT {} START {}", limit, (page - 1) * limit)),
        )
        .await?)
    }

    pub async fn info(&self, id: &str) -> ApiResult<RecordData<User>> {
        RecordIdFunc::from((User::name(), id))
            .get(&*DB)
            .await?
            .ok_or(ApiError::NotFoundInDB)
    }

    pub async fn delete(&self, id: &str) -> ApiResult<()> {
        todo!()
    }

    pub async fn add_achievement(&self, id: &str, achievement: Achievement) -> ApiResult<()> {
        let _: Option<Empty> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::add("/achievements", achievement))
            .await?;
        Ok(())
    }

    pub async fn replace_description(&self, id: &str, description: String) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/bio", description))
            .await?;
        Ok(())
    }

    pub async fn replace_icon_ext(&self, id: &str, icon_ext: &str) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/icon_ext", icon_ext))
            .await?;
        Ok(())
    }

    pub async fn replace_thumb_ext(&self, id: &str, thumb_ext: &str) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/thumb_ext", thumb_ext))
            .await?;
        Ok(())
    }

    pub async fn replace_links(&self, id: &str, links: Vec<String>) -> ApiResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/links", links))
            .await?;
        Ok(())
    }

    pub async fn all_joined_without_achievement(&self) -> Vec<RecordData<UserRolePassword>> {
        todo!()
    }
    pub async fn email_exists(&self, email: &String) -> bool {
        User::search(&*DB, Some(format!("WHERE email == \"{}\" LIMIT 1", email)))
            .await
            .map(|v: Vec<RecordData<Empty>>| !v.is_empty())
            .unwrap_or_default()
    }
    pub async fn get_name_from_ids(
        &self,
        ids: impl Iterator<Item = RecordIdType<User>>,
    ) -> ApiResult<Vec<String>> {
        let v: Vec<RecordData<UserName>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get_part(&*DB)
            .await?;
        Ok(v.into_iter()
            .map(|v| v.data.names.first().cloned().unwrap_or_default())
            .collect())
    }
    pub async fn name_exists(&self, name: &String) -> bool {
        let v: Vec<RecordData<Empty>> = User::search(
            &*DB,
            Some(format!(
                "WHERE array::some(names, |$n: string| string::lowercase($n) = '{name}') LIMIT 1"
            )),
        )
        .await
        .unwrap_or_default();
        !v.is_empty()
    }
    pub async fn set_role(&self, id: &str, role: Role) -> ApiResult<()> {
        let _: Option<Empty> = RecordIdFunc::from((User::name(), id))
            .patch(&*DB, PatchOp::replace("/role", role as u32))
            .await?;
        Ok(())
    }
    pub async fn get_by_name(&self, name: &str) -> ApiResult<RecordData<UserRolePassword>> {
        let name = name.to_lowercase();
        let mut v: Vec<RecordData<UserRolePassword>> = User::search(
            &*DB,
            Some(format!(
                "WHERE array::some(names, |$n: string| string::lowercase($n) = '{name}') LIMIT 1"
            )),
        )
        .await?;
        if v.is_empty() {
            return Err(ApiError::NotFoundInDB);
        }
        Ok(v.remove(0))
    }
    pub async fn get_by_mail(&self, mail: &String) -> ApiResult<RecordData<UserRolePassword>> {
        let mail = mail.to_lowercase();
        let mut v: Vec<RecordData<UserRolePassword>> =
            User::search(&*DB, Some(format!("WHERE email == '{mail}' LIMIT 1"))).await?;
        if v.is_empty() {
            return Err(ApiError::NotFoundInDB);
        }
        Ok(v.remove(0))
    }

    pub async fn get_by_id(&self, id: &str) -> ApiResult<RecordData<UserRolePassword>> {
        let r = RecordIdFunc::from((User::name(), id))
            .get(&*DB)
            .await?
            .ok_or(ApiError::NotFoundInDB)?;
        Ok(r)
    }

    pub async fn get_name_by_id(&self, id: RecordIdType<User>) -> ApiResult<RecordData<String>> {
        let data: RecordData<UserName> = id.get_part(&*DB).await?.ok_or(ApiError::NotFoundInDB)?;
        Ok(RecordData {
            id: data.id,
            data: data.data.names.first().unwrap().clone(),
        })
    }

    pub async fn get_role_and_generated(&self, id: &str) -> ApiResult<(Role, u128)> {
        let data: RecordData<RoleExpiry> = RecordIdFunc::from((User::name(), id))
            .get_part(&*DB)
            .await?
            .ok_or(ApiError::NotFoundInDB)?;
        Ok((
            Role::from(data.data.role),
            data.data
                .generated
                .map(|v| v.into_inner().to_u64())
                .unwrap_or_default()
                .map(Duration::from_nanos)
                .map(|v| v.as_millis())
                .unwrap_or_default(),
        ))
    }

    /// sets a generated timestamp which needs to be older or equal to the current timestamp in the refresh token
    pub async fn logout(&self, id: &str) -> ApiResult<()> {
        todo!()
    }
}
