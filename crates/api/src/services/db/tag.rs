use crate::errors::ApiResult;
use api_structure::models::manga::tag::TagSex;
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
    pub fn to_public(self) -> api_structure::models::manga::tag::Tag {
        api_structure::models::manga::tag::Tag {
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
    pub async fn get_ids(&self, _sex: &Option<u32>, _value: &str) -> ApiResult<ThingArray> {
        todo!()
    }

    pub async fn search_tags(
        &self,
        query: &str,
        limit: usize,
        sex: u64,
    ) -> ApiResult<Vec<RecordData<Tag>>> {
        Ok(Tag::search(
            &self.conn,
            Some(format!(
                "WHERE sex = {sex} AND string::contains(string::lowercase(tag), '{}') LIMIT {limit}"
                , query.to_lowercase().replace("'", "\\'"))),
        )
        .await?)
    }
    pub async fn get_tag(&self, id: &str) -> ApiResult<Option<Tag>> {
        if let Some(v) = self.temp.lock()?.get(id) {
            return Ok(Some(v.clone()));
        }
        let mut hm = HashMap::new();
        let res: Option<Vec<RecordData<Tag>>> = Tag::all(&*self.conn).await.ok();
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
