use serde::{Deserialize, Serialize};
use surrealdb::Datetime;
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealTable, SurrealTableInfo as _,
};

use crate::{
    error::{DbError, DbResult},
    DbSession,
};

use super::manga::vec_default;

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("characters")]
#[sql(["DEFINE EVENT character_updated ON TABLE characters WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Character {
    #[serde(default = "vec_default")]
    pub imgs: Vec<Option<String>>,
    #[serde(default = "vec_default")]
    pub names: Vec<String>,
    pub description: Option<String>,
    pub sex: Option<u64>,
    #[serde(default = "vec_default")]
    pub links: Vec<String>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(Clone)]
pub struct CharacterDBService {
    db: DbSession,
}

impl CharacterDBService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn create(&self, character: Character) -> DbResult<RecordIdType<Character>> {
        character
            .add(self.db.as_ref())
            .await?
            .map(|item| item.id.into())
            .ok_or(DbError::NotFound)
    }

    pub async fn info(&self, id: &str) -> DbResult<RecordData<Character>> {
        RecordIdFunc::from((Character::name(), id))
            .get_part(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
        limit: u32,
    ) -> DbResult<Vec<RecordData<Character>>> {
        let start = page.saturating_sub(1) * limit;
        let sql = format!(
            "SELECT * FROM {} WHERE names.any(|$s|string::contains(string::lowercase($s), $query)) LIMIT $limit START $start",
            Character::name()
        );
        let values: Vec<RecordData<Character>> = self
            .db
            .query(sql)
            .bind(("query", query.to_lowercase()))
            .bind(("limit", limit))
            .bind(("start", start))
            .await?
            .take(0)?;
        Ok(values)
    }
}
