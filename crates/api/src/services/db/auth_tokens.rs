use crate::env::config::random_string;
use crate::errors::{ApiError, ApiResult};
use crate::services::db::user::User;
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::models::auth::kind::Kind;
use api_structure::models::auth::role::Role;
use api_structure::now_timestamp;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use surrealdb_extras::{RecordData, SurrealSelect, SurrealTable, SurrealTableInfo, ThingType};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("auth_tokens")]
#[sql(["INSERT INTO auth_tokens {
    user: NONE,
    token: '000000',
    kind: 51,
    active_until_timestamp: 253370764800000,
};"])]
pub struct AuthToken {
    user: Option<ThingType<User>>,
    token: String,
    kind: u32,
    active_until_timestamp: u64,
}

impl AuthToken {
    pub fn new_forgot(user_id: String) -> ApiResult<Self> {
        Ok(Self {
            user: Some(ThingType::from(Thing::from(("users", user_id.as_str())))),
            token: random_string(6),
            kind: Kind {
                single: true,
                kind: Role::NotVerified,
            }
            .into(),
            active_until_timestamp: (now_timestamp().map_err(|e| ApiError::Inner(e))?
                + Duration::from_secs(3600))
            .as_millis() as u64,
        })
    }
}

#[derive(SurrealSelect, Deserialize)]
pub struct AuthUser {
    pub user: Option<ThingType<User>>,
    kind: u64,
}

impl AuthUser {
    pub fn get_kind(&self) -> Kind {
        Kind::from(self.kind as u32)
    }
}

pub struct AuthTokenDBService {
    pub conn: Arc<Surreal<Db>>,
}

impl AuthTokenDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }

    pub async fn check(&self, token: &str) -> ApiResult<RecordData<AuthUser>> {
        let query = format!(
            "WHERE token = \"{}\" AND active_until_timestamp >= {}",
            token,
            now_timestamp().map_err(ApiError::Inner)?.as_millis()
        );

        let mut search = AuthToken::search(&*self.conn, Some(query)).await?;
        if search.is_empty() {
            return Err(ApiErr {
                message: Some("Not valid token".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }
            .into());
        }
        Ok(search.remove(0))
    }
}
