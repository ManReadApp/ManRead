use crate::errors::{ApiError, ApiResult};
use image::DynamicImage;
use img_hash::Hasher;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use surrealdb_extras::{RecordData, SurrealTable, ThingType};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("manga_pages")]
#[sql(["DEFINE EVENT manga_page_updated ON TABLE manga_pages WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Page {
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub ext: String,
    pub hash: String,
    pub translation: bool,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

impl Page {
    #[allow(dead_code)]
    pub fn new(img: DynamicImage, ext: &str, page: u32, hasher: &Hasher) -> Self {
        let hash = hasher.hash_image(&img).to_base64();
        Self {
            page,
            width: img.width(),
            height: img.height(),
            ext: ext.to_string(),
            translation: false,
            hash,
            updated: Default::default(),
            created: Default::default(),
        }
    }
}

pub struct PageDBService {
    conn: Arc<Surreal<Db>>,
}

impl PageDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }

    pub async fn get(&self, page: ThingType<Page>) -> ApiResult<Page> {
        let v: RecordData<Page> = page
            .get_part(&*self.conn)
            .await?
            .ok_or(ApiError::db_error())?;
        Ok(v.data)
    }
}
