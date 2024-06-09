use crate::services::db::chapter_version::ChapterVersion;
use crate::services::db::scrape_account::ScrapeAccount;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use surrealdb_extras::{SurrealTable, ThingType};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("scrape_list")]
pub struct ScrapeItem {
    scraper: u32,
    chapter_version: ThingType<ChapterVersion>,
    scrape_account: Option<ThingType<ScrapeAccount>>,
    url: String,
    download_timestamp: u64,
    info: String,
}

pub struct ScrapeListDBService {
    #[allow(dead_code)]
    conn: Arc<Surreal<Db>>,
}

impl ScrapeListDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }
}
