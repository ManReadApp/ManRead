use serde::{Deserialize, Serialize};
use surrealdb::Datetime;
use surrealdb_extras::{RecordIdType, SurrealTable, SurrealTableInfo};

use crate::{
    error::{DbError, DbResult},
    DB,
};

use super::{page::Page, version::Version};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("chapter_version_connections")]
#[sql(["DEFINE EVENT chapter_version_conn_updated ON TABLE chapter_version_connections WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct ChapterVersion {
    /// Version
    pub version: RecordIdType<Version>,
    /// Page ids which are in a chapter
    pub pages: Vec<RecordIdType<Page>>,
    /// Link to external website to read the chapter
    pub link: Option<String>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(Default)]
pub struct ChapterVersionDBService {}

impl ChapterVersionDBService {
    pub async fn get(&self, id: &str) -> DbResult<ChapterVersion> {
        let item = RecordIdType::from((ChapterVersion::name(), id))
            .get(&*DB)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(item.data)
    }
}
