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
    pub async fn get_id(&self, _kind: &str) -> ApiResult<ThingFunc> {
        todo!()
    }
    pub async fn get_kind(&self, id: &str) -> Option<Kind> {
        if let Some(v) = self.temp.lock().unwrap().get(id) {
            return Some(v.clone());
        }
        let mut hm = HashMap::new();
        let res: Vec<RecordData<Kind>> = Kind::all(&*self.conn).await.ok()?;
        for item in res {
            hm.insert(item.id.id().to_string(), item.data);
        }
        let v = hm.get(id).cloned();
        *self.temp.lock().unwrap() = hm;
        v
    }

    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self {
            conn,
            temp: Default::default(),
        }
    }
}
