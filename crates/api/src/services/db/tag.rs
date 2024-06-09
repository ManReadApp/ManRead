use crate::errors::ApiResult;
use api_structure::info::TagSex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use surrealdb_extras::{RecordData, SurrealTable, SurrealTableInfo, ThingArray};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("tags")]
#[sql(["DEFINE EVENT tag_updated ON TABLE tags WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Tag {
    pub tag: String,
    pub description: Option<String>,
    pub sex: u64,
}

impl Tag {
    pub fn to_public(self) -> api_structure::info::Tag {
        api_structure::info::Tag {
            tag: self.tag,
            description: self.description,
            sex: TagSex::from(self.sex),
        }
    }
}

pub struct TagDBService {
    pub conn: Arc<Surreal<Db>>,
    temp: Arc<Mutex<HashMap<String, Tag>>>,
}

impl TagDBService {
    pub async fn get_ids(&self, sex: &Option<u32>, value: &str) -> ApiResult<ThingArray> {
        todo!()
    }
    pub async fn get_tag(&self, id: &str) -> Option<Tag> {
        if let Some(v) = self.temp.lock().unwrap().get(id) {
            return Some(v.clone());
        }
        let mut hm = HashMap::new();
        let res: Vec<RecordData<Tag>> = Tag::all(&*self.conn).await.ok()?;
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
