use serde::{Deserialize, Serialize};
use storage::{FileBuilderExt as _, MangaPageFileBuilder};
use surrealdb::Datetime;
use surrealdb_extras::{RecordData, RecordIdType, SurrealTable, ThingArray};

use crate::{
    error::{DbError, DbResult},
    DbSession,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("manga_pages")]
#[sql(["DEFINE EVENT manga_page_updated ON TABLE manga_pages WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Page {
    /// Page number
    pub page: u32,
    /// Width of the page
    pub width: u32,
    /// Height of the page
    pub height: u32,
    /// Extension of the page
    pub ext: String,
    /// Hash of the page
    pub hash: Option<String>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(Clone)]
pub struct PageDBService {
    db: DbSession,
}

impl Default for PageDBService {
    fn default() -> Self {
        Self::new(crate::global_db())
    }
}

impl PageDBService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn get(&self, ids: Vec<RecordIdType<Page>>) -> DbResult<Vec<RecordData<Page>>> {
        let out: Vec<RecordData<Page>> = ThingArray::from(ids).get(self.db.as_ref()).await?;
        Ok(out)
    }
    pub async fn add(&self, pages: Vec<MangaPageFileBuilder>) -> DbResult<Vec<RecordIdType<Page>>> {
        let mut out = vec![];
        for (index, page) in pages.into_iter().enumerate() {
            let ext = page.ext().map_err(|_| DbError::NoExtension)?.to_owned();
            let p = Page {
                page: index as u32 + 1,
                width: page.width().ok_or(DbError::NoImage)?,
                height: page.height().ok_or(DbError::NoImage)?,
                ext: ext.clone(),
                hash: None,
                updated: Default::default(),
                created: Default::default(),
            }
            .add(self.db.as_ref())
            .await?;
            let p = p.ok_or(DbError::NotFound)?;

            let enc = page.build(index + 1).await?;
            out.push(p);
        }
        Ok(out.into_iter().map(Into::into).collect())
    }
}
