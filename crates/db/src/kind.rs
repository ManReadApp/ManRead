use serde::{Deserialize, Serialize};
use surrealdb_extras::{RecordData, RecordIdType, SurrealTable, SurrealTableInfo};

use crate::{
    error::{DbError, DbResult},
    DB,
};

use super::tag::Empty;

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("kinds")]
pub struct Kind {
    /// Manga kind like manga, manhwa, manhua, etc.
    pub kind: String,
}

#[derive(Default)]
pub struct KindDBService {}

impl KindDBService {
    pub async fn all(&self) -> DbResult<Vec<String>> {
        let v: Vec<RecordData<Kind>> = Kind::all(&*DB).await?;
        Ok(v.into_iter().map(|v| v.data.kind).collect())
    }

    pub async fn get_or_create(&self, kind: &str) -> DbResult<RecordIdType<Kind>> {
        let mut v: Vec<RecordData<Empty>> =
            Kind::search(&*DB, Some(format!("WHERE kind = '{}'", kind))).await?;
        if v.is_empty() {
            let v = Kind {
                kind: kind.to_owned(),
            }
            .add_i(&*DB)
            .await?;
            Ok(v.id.into())
        } else {
            Ok(v.remove(0).id.into())
        }
    }

    pub async fn get_name(&self, id: RecordIdType<Kind>) -> DbResult<String> {
        Ok(id.get(&*DB).await?.ok_or(DbError::NotFound)?.data.kind)
    }
}
