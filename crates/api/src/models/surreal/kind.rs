use serde::{Deserialize, Serialize};
use surrealdb_extras::{RecordData, RecordIdType, SurrealTable, SurrealTableInfo};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
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
    pub async fn all(&self) -> ApiResult<Vec<String>> {
        let v: Vec<RecordData<Kind>> = Kind::all(&*DB).await?;
        Ok(v.into_iter().map(|v| v.data.kind).collect())
    }

    pub async fn get_or_create(&self, kind: &str) -> ApiResult<RecordIdType<Kind>> {
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

    pub async fn get_name(&self, id: RecordIdType<Kind>) -> ApiResult<String> {
        Ok(id.get(&*DB).await?.ok_or(ApiError::NotFoundInDB)?.data.kind)
    }
}
