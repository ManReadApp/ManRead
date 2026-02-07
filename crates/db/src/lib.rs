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

use std::sync::{Arc, LazyLock};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};
pub use surrealdb_extras::{RecordIdFunc, RecordIdType};

use crate::auth::AuthTokenDBService;
use crate::user::UserDBService;

pub static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

pub struct DbConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8083,
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct DbHandle {
    pub tokens: Arc<AuthTokenDBService>,
    pub users: Arc<UserDBService>,
}

pub async fn init_db(config: DbConfig) -> Result<DbHandle, surrealdb::Error> {
    DB.connect::<Ws>(&format!("{}:{}", config.host, config.port))
        .await?;

    DB.signin(Root {
        username: &config.username,
        password: &config.password,
    })
    .await?;

    DB.use_ns("manread").use_db("manread").await?;
    Ok(DbHandle {
        users: Arc::default(),
        tokens: Arc::default(),
    })
}
