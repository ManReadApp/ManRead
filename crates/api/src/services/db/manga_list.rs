use crate::services::db::manga::Manga;
use crate::services::db::user::User;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use surrealdb_extras::{SurrealTable, ThingType};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("manga_lists")]
#[sql(["DEFINE EVENT manga_list_updated ON TABLE manga_lists WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct MangaList {
    name: String,
    user: ThingType<User>,
    mangas: HashSet<ThingType<Manga>>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

impl MangaList {
    #[allow(dead_code)]
    fn new(name: String, user: ThingType<User>) -> Self {
        Self {
            name,
            user,
            mangas: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        }
    }
}

pub struct MangaListDBService {
    #[allow(dead_code)]
    conn: Arc<Surreal<Db>>,
}

impl MangaListDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }
}
