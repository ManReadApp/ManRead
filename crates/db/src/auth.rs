use api_structure::models::auth::kind::TokenKind;
use helper::random_string;
use serde::{Deserialize, Serialize};
pub use surrealdb_extras::RecordData;
use surrealdb_extras::RecordIdFunc;
use surrealdb_extras::RecordIdType;
use surrealdb_extras::SurrealTable;
use surrealdb_extras::SurrealTableInfo as _;

use crate::error::DbError;
use crate::error::DbResult;
use crate::DB;

use super::user::User;

#[derive(Default)]
pub struct AuthTokenDBService {}

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
    pub fn get_kind(&self) -> TokenKind {
        TokenKind::try_from(self.kind as u32).unwrap()
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
    pub async fn list(&self, page: u32, limit: u32) -> DbResult<Vec<RecordData<AuthUser>>> {
        let offset = (page - 1) * limit;
        let search: Vec<RecordData<AuthUser>> =
            AuthUser::search(&*DB, Some(format!("LIMIT {limit} START {offset}"))).await?;
        Ok(search)
    }
    pub async fn delete(&self, id: &str) -> DbResult<()> {
        RecordIdFunc::from((AuthUser::name(), id))
            .delete_s(&*DB)
            .await?;
        Ok(())
    }
    pub async fn find(&self, token: &str) -> DbResult<RecordData<AuthUser>> {
        let query = format!("WHERE token = \"{token}\"",);

        let mut search: Vec<RecordData<AuthUser>> = AuthUser::search(&*DB, Some(query)).await?;
        if search.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(search.remove(0))
    }
    pub async fn create(&self, user_id: Option<String>, kind: TokenKind) -> DbResult<()> {
        let user = user_id.map(|v| RecordIdType::from((User::name(), v.as_str())));
        AuthUser {
            user,
            token: random_string(6),
            kind: u32::from(kind),
            active_until_timestamp: None,
        }
        .add_i(&*DB)
        .await?;
        Ok(())
    }
}
