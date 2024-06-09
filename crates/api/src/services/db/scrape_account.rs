use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use surrealdb_extras::SurrealTable;

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("scrape_accounts")]
pub struct ScrapeAccount {
    pub username: String,
    pub password: String,
    pub site: String,
    pub exclude: Vec<String>,
}

pub struct ScrapeAccountDBService {
    #[allow(dead_code)]
    conn: Arc<Surreal<Db>>,
}

impl ScrapeAccountDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }
}
