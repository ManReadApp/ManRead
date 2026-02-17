use api_structure::v1::{ActivationTokenKind, Role};
use helper::random_string;
use serde::{Deserialize, Serialize};
pub use surrealdb_extras::RecordData;
use surrealdb_extras::RecordIdFunc;
use surrealdb_extras::RecordIdType;
use surrealdb_extras::SurrealTable;
use surrealdb_extras::SurrealTableInfo as _;

use crate::error::DbError;
use crate::error::DbResult;
use crate::DbSession;

use super::user::User;

#[derive(Clone)]
pub struct AuthTokenDBService {
    db: DbSession,
}

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("auth_tokens")]
#[sql(["INSERT INTO auth_tokens {
    user: NONE,
    token: '000000',
    kind: 61,
    active_until_timestamp: NONE,
};"])]
pub struct AuthUser {
    /// Limits who can use the token
    pub user: Option<RecordIdType<User>>,
    /// TokenKind defines if the token is single use or not & if it is used to reset a password or verify a user
    token: String,
    kind: u32,
    pub active_until_timestamp: Option<u64>,
}

impl AuthUser {
    pub fn get_kind(&self) -> ActivationTokenKind {
        ActivationTokenKind::try_from(self.kind).unwrap_or(ActivationTokenKind {
            single: true,
            kind: Role::NotVerified,
        })
    }
}

pub fn is_token_valid(token: &RecordData<AuthUser>, user_id: &str) -> DbResult<()> {
    if let Some(v) = &token.data.user {
        if v.id().to_string() != user_id {
            return Err(DbError::InvalidActivationToken);
        }
    }
    if let Some(max_age) = token.data.active_until_timestamp {
        if max_age < chrono::Utc::now().timestamp() as u64 {
            return Err(DbError::ExpiredToken);
        }
    }
    Ok(())
}

impl AuthTokenDBService {
    pub async fn delete_(&self, token: RecordData<AuthUser>) -> DbResult<()> {
        token
            .delete_s(self.db.as_ref())
            .await
            .map_err(DbError::from)?;
        Ok(())
    }
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn list(&self, page: u32, limit: u32) -> DbResult<Vec<RecordData<AuthUser>>> {
        let offset = page.saturating_sub(1) * limit;
        let search: Vec<RecordData<AuthUser>> = AuthUser::search(
            self.db.as_ref(),
            Some(format!("LIMIT {limit} START {offset}")),
        )
        .await?;
        Ok(search)
    }
    pub async fn delete(&self, id: &str) -> DbResult<()> {
        RecordIdFunc::from((AuthUser::name(), id))
            .delete_s(self.db.as_ref())
            .await?;
        Ok(())
    }
    pub async fn find(&self, token: &str) -> DbResult<RecordData<AuthUser>> {
        let query = format!("WHERE token = \"{token}\"",);

        let mut search: Vec<RecordData<AuthUser>> =
            AuthUser::search(self.db.as_ref(), Some(query)).await?;
        if search.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(search.remove(0))
    }
    pub async fn create(&self, user_id: Option<String>, kind: ActivationTokenKind) -> DbResult<()> {
        let user = user_id.map(|v| RecordIdType::from((User::name(), v.as_str())));
        AuthUser {
            user,
            token: random_string(6),
            kind: u32::from(kind),
            active_until_timestamp: None,
        }
        .add_i(self.db.as_ref())
        .await?;
        Ok(())
    }
}
