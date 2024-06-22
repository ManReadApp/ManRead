use crate::errors::{ApiError, ApiResult};
use crate::services::db::chapter_version::ChapterVersion;
use crate::services::db::tag::Tag;
use api_structure::models::reader::chapter::ReaderChapter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use surrealdb_extras::{RecordData, SurrealSelect, SurrealTable, ThingType};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("chapters")]
#[sql(["DEFINE EVENT chapter_updated ON TABLE chapters WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Chapter {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<ThingType<Tag>>,
    pub sources: Vec<String>,
    pub release_date: Option<Datetime>,
    ///Version to string
    pub versions: HashMap<String, ThingType<ChapterVersion>>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(SurrealSelect, Deserialize)]
pub struct ChapterReaderPart {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub sources: Vec<String>,
    pub release_date: Option<Datetime>,
    pub versions: HashMap<String, ThingType<ChapterVersion>>,
}

pub struct ChapterDBService {
    conn: Arc<Surreal<Db>>,
}

impl ChapterDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }

    pub async fn get_reader(&self, id: ThingType<Chapter>) -> ApiResult<ReaderChapter> {
        let res: RecordData<ChapterReaderPart> = id
            .get_part(&*self.conn)
            .await?
            .ok_or(ApiError::db_error())?;
        Ok(ReaderChapter {
            chapter_id: res.id.id().to_string(),
            titles: res.data.titles,
            chapter: res.data.chapter,
            sources: res.data.sources,
            release_date: res.data.release_date.map(|v| v.to_string()),
            versions: res
                .data
                .versions
                .into_iter()
                .map(|(key, value)| (key, value.thing.id().to_string()))
                .collect(),
        })
    }
}
