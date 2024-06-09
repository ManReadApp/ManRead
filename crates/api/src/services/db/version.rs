use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use surrealdb_extras::SurrealTable;

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("chapter_versions")]
pub struct Version {
    pub name: String,
    pub translate_opts: Option<String>,
}

impl Version {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self {
            name,
            translate_opts: None,
        }
    }
}

pub struct VersionDBService {
    #[allow(dead_code)]
    conn: Arc<Surreal<Db>>,
}

impl VersionDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }
}
