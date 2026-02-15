use std::time::Duration;

use api_structure::v1::{Role, Tag};
use chrono::{DateTime, Utc};
use helper::random_string;
use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo as _,
    ThingArray,
};

use crate::{
    error::{DbError, DbResult},
    tag::Empty,
    DbSession,
};

use super::manga::vec_default;

#[derive(Clone)]
pub struct UserDBService {
    db: DbSession,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Achievement {
    Joined,
    // 10, 100, 200, 500, 1000
    Read(u32),
    // 50, 100, 200, 500, 1000
    Favorited(u32),
    // 100, 200, 500, 1000, 10000
    Commented(u32),
    // 20, 100, 200, 500, 1000
    Reviewed(u32),
}

impl From<u64> for Achievement {
    fn from(value: u64) -> Self {
        let tag = (value >> 32) as u32;
        let payload = (value & 0xFFFF_FFFF) as u32;

        match tag {
            TAG_JOINED => Achievement::Joined,
            TAG_READ => Achievement::Read(payload),
            TAG_FAVORITED => Achievement::Favorited(payload),
            TAG_COMMENTED => Achievement::Commented(payload),
            TAG_REVIEWED => Achievement::Reviewed(payload),
            other => todo!("Unknown tag: {}", other),
        }
    }
}
const TAG_JOINED: u32 = 0;
const TAG_READ: u32 = 1;
const TAG_FAVORITED: u32 = 2;
const TAG_COMMENTED: u32 = 3;
const TAG_REVIEWED: u32 = 4;
impl From<Achievement> for u64 {
    fn from(value: Achievement) -> Self {
        let (tag, payload): (u32, u32) = match value {
            Achievement::Joined => (TAG_JOINED, 0),
            Achievement::Read(v) => (TAG_READ, v),
            Achievement::Favorited(v) => (TAG_FAVORITED, v),
            Achievement::Commented(v) => (TAG_COMMENTED, v),
            Achievement::Reviewed(v) => (TAG_REVIEWED, v),
        };

        ((tag as u64) << 32) | (payload as u64)
    }
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

impl Default for UserDBService {
    fn default() -> Self {
        Self::new(crate::global_db())
    }
}

impl UserDBService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn get_or_create(&self, name: &str) -> DbResult<RecordIdFunc> {
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
    ) -> DbResult<Vec<RecordData<SimpleUser>>> {
        let user: Vec<RecordData<SimpleUser>> = User::search(
            self.db.as_ref(),
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
    ) -> DbResult<RecordData<UserRolePassword>> {
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
        .add(self.db.as_ref())
        .await?
        .ok_or(DbError::NotFound)?;
        Ok(RecordData {
            id: user.id,
            data: UserRolePassword {
                role: user.data.role,
                email: user.data.email.to_lowercase(),
                password: user.data.password,
            },
        })
    }

    pub async fn replace_names(&self, id: &str, names: Vec<String>) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/names", names))
            .await?;
        Ok(())
    }

    pub async fn set_password(&self, id: &str, password: String) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/password", password))
            .await?;
        Ok(())
    }

    pub async fn list(&self, page: u32, limit: u32) -> DbResult<Vec<RecordData<SimpleUser>>> {
        Ok(User::search(
            self.db.as_ref(),
            Some(format!("LIMIT {} START {}", limit, (page - 1) * limit)),
        )
        .await?)
    }

    pub async fn info(&self, id: &str) -> DbResult<RecordData<User>> {
        RecordIdFunc::from((User::name(), id))
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> DbResult<()> {
        todo!("mark only")
    }

    pub async fn add_achievement(&self, id: &str, achievement: Achievement) -> DbResult<()> {
        let _: Option<Empty> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::add("/achievements", achievement))
            .await?;
        Ok(())
    }

    pub async fn replace_description(&self, id: &str, description: String) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/bio", description))
            .await?;
        Ok(())
    }

    pub async fn replace_icon_ext(&self, id: &str, icon_ext: &str) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/icon_ext", icon_ext))
            .await?;
        Ok(())
    }

    pub async fn replace_thumb_ext(&self, id: &str, thumb_ext: &str) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/thumb_ext", thumb_ext))
            .await?;
        Ok(())
    }

    pub async fn replace_links(&self, id: &str, links: Vec<String>) -> DbResult<()> {
        let _: Option<RecordData<Empty>> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/links", links))
            .await?;
        Ok(())
    }

    pub async fn all_joined_without_achievement(&self) -> Vec<RecordData<UserRolePassword>> {
        todo!()
    }
    pub async fn email_exists(&self, email: &String) -> bool {
        User::search(
            self.db.as_ref(),
            Some(format!("WHERE email == \"{}\" LIMIT 1", email)),
        )
        .await
        .map(|v: Vec<RecordData<Empty>>| !v.is_empty())
        .unwrap_or_default()
    }
    pub async fn get_name_from_ids(
        &self,
        ids: impl Iterator<Item = RecordIdType<User>>,
    ) -> DbResult<Vec<String>> {
        let v: Vec<RecordData<UserName>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get_part(self.db.as_ref())
            .await?;
        Ok(v.into_iter()
            .map(|v| v.data.names.first().cloned().unwrap_or_default())
            .collect())
    }
    pub async fn name_exists(&self, name: &String) -> bool {
        let v: Vec<RecordData<Empty>> = User::search(
            self.db.as_ref(),
            Some(format!(
                "WHERE array::some(names, |$n: string| string::lowercase($n) = '{name}') LIMIT 1"
            )),
        )
        .await
        .unwrap_or_default();
        !v.is_empty()
    }
    pub async fn set_role(&self, id: &str, role: Role) -> DbResult<()> {
        let _: Option<Empty> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/role", role as u32))
            .await?;
        Ok(())
    }
    pub async fn get_by_name(&self, name: &str) -> DbResult<RecordData<UserRolePassword>> {
        let name = name.to_lowercase();
        let mut v: Vec<RecordData<UserRolePassword>> = User::search(
            self.db.as_ref(),
            Some(format!(
                "WHERE array::some(names, |$n: string| string::lowercase($n) = '{name}') LIMIT 1"
            )),
        )
        .await?;
        if v.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(v.remove(0))
    }
    pub async fn get_by_mail(&self, mail: &String) -> DbResult<RecordData<UserRolePassword>> {
        let mail = mail.to_lowercase();
        let mut v: Vec<RecordData<UserRolePassword>> = User::search(
            self.db.as_ref(),
            Some(format!("WHERE email == '{mail}' LIMIT 1")),
        )
        .await?;
        if v.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(v.remove(0))
    }

    pub async fn get_by_id(&self, id: &str) -> DbResult<RecordData<UserRolePassword>> {
        let r = RecordIdFunc::from((User::name(), id))
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(r)
    }

    pub async fn get_name_by_id(&self, id: RecordIdType<User>) -> DbResult<RecordData<String>> {
        let data: RecordData<UserName> = id
            .get_part(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(RecordData {
            id: data.id,
            data: data.data.names.first().unwrap().clone(),
        })
    }

    pub async fn get_role_and_generated(&self, id: &str) -> DbResult<(Role, u128)> {
        let data: RecordData<RoleExpiry> = RecordIdFunc::from((User::name(), id))
            .get_part(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;
        Ok((
            Role::try_from(data.data.role).unwrap(),
            data.data
                .generated
                .map(|v| v.into_inner().to_u64())
                .unwrap_or_default()
                .map(Duration::from_nanos)
                .map(|v| v.as_millis())
                .unwrap_or_default(),
        ))
    }

    /// sets a generated timestamp which needs to be older or equal to the current timestamp in the refresh token to be valid
    pub async fn logout(&self, id: &str) -> DbResult<()> {
        let dt = Datetime::default();
        let _: Option<Empty> = RecordIdFunc::from((User::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/generated", dt))
            .await?;
        Ok(())
    }
}
