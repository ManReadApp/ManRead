use serde::{Deserialize, Serialize};
use surrealdb::opt::PatchOp;
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealTable, SurrealTableInfo as _,
};

use crate::{
    error::{DbError, DbResult},
    DbSession,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[db("chapter_versions")]
pub struct Version {
    /// Version of a chapter like japanese_black_and_white, english_colored, etc.
    pub name: String,
    /// Query to send to the translator
    pub translate_opts: Option<String>,
}

#[derive(Clone)]
pub struct VersionDBService {
    db: DbSession,
}

impl Default for VersionDBService {
    fn default() -> Self {
        Self::new(crate::global_db())
    }
}

impl VersionDBService {
    pub async fn info(&self, id: RecordIdType<Version>) -> DbResult<RecordData<Version>> {
        id.get(self.db.as_ref()).await?.ok_or(DbError::NotFound)
    }

    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn rename(&self, id: &str, name: String) -> DbResult<()> {
        let _: Option<Version> = RecordIdFunc::from((Version::name(), id))
            .patch(self.db.as_ref(), PatchOp::replace("/name", name))
            .await?;
        Ok(())
    }
    pub async fn update_translate_opts(&self, id: &str, translate_opts: String) -> DbResult<()> {
        let _: Option<Version> = RecordIdFunc::from((Version::name(), id))
            .patch(
                self.db.as_ref(),
                PatchOp::replace("/translate_opts", translate_opts),
            )
            .await?;
        Ok(())
    }

    pub async fn list(&self, page: u32, limit: u32) -> DbResult<Vec<RecordData<Version>>> {
        let offset = (page - 1) * limit;
        let search: Vec<RecordData<Version>> = Version::search(
            self.db.as_ref(),
            Some(format!("LIMIT {limit} START {offset}")),
        )
        .await?;
        Ok(search)
    }

    pub async fn get(&self, version: &str) -> DbResult<RecordIdType<Version>> {
        let mut search: Vec<RecordData<Version>> = Version::search(
            self.db.as_ref(),
            Some(format!("WHERE name = \"{version}\"")),
        )
        .await?;
        if search.is_empty() {
            return Ok(Version {
                name: version.to_owned(),
                translate_opts: None,
            }
            .add_i(self.db.as_ref())
            .await?
            .id
            .into());
        }
        Ok(search.remove(0).id.into())
    }
}
