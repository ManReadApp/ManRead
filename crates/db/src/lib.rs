pub mod auth;
pub mod chapter;
pub mod error;
pub mod kind;
pub mod lists;
pub mod logs;
pub mod manga;
pub mod page;
pub mod progress;
pub mod scraper;
pub mod tag;
pub mod user;
pub mod version;
pub mod version_link;
use std::sync::{Arc, OnceLock};
pub use surrealdb::RecordId;
pub use surrealdb_extras::SurrealTableInfo;

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use surrealdb::opt::auth::Root;
pub use surrealdb_extras::{RecordIdFunc, RecordIdType};

use crate::auth::AuthTokenDBService;
use crate::user::UserDBService;

pub type DbClient = Surreal<Any>;
pub type DbSession = Arc<DbClient>;
static GLOBAL_DB: OnceLock<DbSession> = OnceLock::new();

#[derive(Clone)]
pub struct RemoteDbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Clone)]
pub struct MemoryDbConfig {
    pub namespace: String,
    pub database: String,
}

#[derive(Clone)]
pub enum DbConfig {
    Remote(RemoteDbConfig),
    Memory(MemoryDbConfig),
}

impl Default for DbConfig {
    fn default() -> Self {
        Self::Remote(RemoteDbConfig {
            host: "localhost".to_string(),
            port: 8083,
            username: "root".to_string(),
            password: "root".to_string(),
            namespace: "manread".to_string(),
            database: "manread".to_string(),
        })
    }
}

#[derive(Clone)]
pub struct DbHandle {
    pub session: DbSession,
    pub tokens: Arc<AuthTokenDBService>,
    pub users: Arc<UserDBService>,
}

pub async fn init_db(config: DbConfig) -> Result<DbHandle, surrealdb::Error> {
    let db = Arc::new(Surreal::init());
    match &config {
        DbConfig::Remote(cfg) => {
            DbClient::connect(db.as_ref(), format!("ws://{}:{}", cfg.host, cfg.port)).await?;
            db.signin(Root {
                username: &cfg.username,
                password: &cfg.password,
            })
            .await?;
            db.use_ns(&cfg.namespace).use_db(&cfg.database).await?;
        }
        DbConfig::Memory(cfg) => {
            DbClient::connect(db.as_ref(), "mem://").await?;
            db.use_ns(&cfg.namespace).use_db(&cfg.database).await?;
        }
    }

    let _ = GLOBAL_DB.set(db.clone());
    Ok(DbHandle {
        session: db.clone(),
        users: Arc::new(UserDBService::new(db.clone())),
        tokens: Arc::new(AuthTokenDBService::new(db)),
    })
}

pub(crate) fn global_db() -> DbSession {
    GLOBAL_DB
        .get()
        .cloned()
        .expect("Database not initialized. Call init_db() before using Default services.")
}
