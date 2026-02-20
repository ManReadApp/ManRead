pub mod auth;
pub mod chapter;
pub mod character;
pub mod error;
pub mod kind;
pub mod lists;
pub mod manga;
pub mod page;
pub mod progress;
pub mod scraper;
pub mod tag;
pub mod user;
pub mod version;
pub mod version_link;
use std::sync::Arc;
pub use surrealdb::RecordId;
pub use surrealdb_extras::SurrealTableInfo;

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use surrealdb::opt::auth::Root;
pub use surrealdb_extras::{RecordIdFunc, RecordIdType};

use crate::auth::AuthTokenDBService;
use crate::chapter::ChapterDBService;
use crate::character::CharacterDBService;
use crate::kind::KindDBService;
use crate::lists::ListDBService;
use crate::manga::MangaDBService;
use crate::page::PageDBService;
use crate::progress::UserProgressDBService;
use crate::scraper::ScraperDbService;
use crate::tag::TagDBService;
use crate::user::UserDBService;
use crate::version::VersionDBService;
use crate::version_link::ChapterVersionDBService;

pub type DbClient = Surreal<Any>;
pub type DbSession = Arc<DbClient>;

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
    pub characters: Arc<CharacterDBService>,
    pub chapters: Arc<ChapterDBService>,
    pub kinds: Arc<KindDBService>,
    pub lists: Arc<ListDBService>,
    pub mangas: Arc<MangaDBService>,
    pub pages: Arc<PageDBService>,
    pub progress: Arc<UserProgressDBService>,
    pub scraper: Arc<ScraperDbService>,
    pub tags: Arc<TagDBService>,
    pub versions: Arc<VersionDBService>,
    pub chapter_versions: Arc<ChapterVersionDBService>,
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

    Ok(DbHandle {
        session: db.clone(),
        users: Arc::new(UserDBService::new(db.clone())),
        tokens: Arc::new(AuthTokenDBService::new(db.clone())),
        characters: Arc::new(CharacterDBService::new(db.clone())),
        chapters: Arc::new(ChapterDBService::new(db.clone())),
        kinds: Arc::new(KindDBService::new(db.clone())),
        lists: Arc::new(ListDBService::new(db.clone())),
        mangas: Arc::new(MangaDBService::new(db.clone())),
        pages: Arc::new(PageDBService::new(db.clone())),
        progress: Arc::new(UserProgressDBService::new(db.clone())),
        scraper: Arc::new(ScraperDbService::new(db.clone())),
        tags: Arc::new(TagDBService::new(db.clone())),
        versions: Arc::new(VersionDBService::new(db.clone())),
        chapter_versions: Arc::new(ChapterVersionDBService::new(db)),
    })
}
