use crate::errors::ApiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use surrealdb_extras::{RecordData, SurrealTable, SurrealTableInfo, ThingFunc};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("kinds")]
pub struct Kind {
    pub kind: String,
}

pub struct MangaKindDBService {
    conn: Arc<Surreal<Db>>,
    temp: Arc<Mutex<HashMap<String, Kind>>>,
}

impl MangaKindDBService {
    pub async fn all(&self) -> ApiResult<Vec<String>> {
        let items: Vec<RecordData<Kind>> = Kind::all(&self.conn).await?;
        Ok(items.into_iter().map(|v| v.data.kind).collect())
    }
    pub async fn get_id(&self, _kind: &str) -> ApiResult<ThingFunc> {
        todo!()
    }
    pub async fn get_kind(&self, id: &str) -> ApiResult<Option<Kind>> {
        if let Some(v) = self.temp.lock()?.get(id) {
            return Ok(Some(v.clone()));
        }
        let mut hm = HashMap::new();
        let res: Option<Vec<RecordData<Kind>>> = Kind::all(&*self.conn).await.ok();
        let res = match res {
            Some(v) => v,
            None => return Ok(None),
        };
        for item in res {
            hm.insert(item.id.id().to_string(), item.data);
        }
        let v = hm.get(id).cloned();
        *self.temp.lock()? = hm;
        Ok(v)
    }

    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self {
            conn,
            temp: Default::default(),
        }
    }
}
