use serde::{Deserialize, Serialize};
use surrealdb::opt::PatchOp;
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealTable, SurrealTableInfo as _,
};

use crate::{error::ApiResult, init::db::DB};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[db("chapter_versions")]
pub struct Version {
    /// Version of a chapter like japanese_black_and_white, english_colored, etc.
    pub name: String,
    /// Query to send to the translator
    pub translate_opts: Option<String>,
}

#[derive(Default)]
pub struct VersionDBService {}

impl VersionDBService {
    pub async fn rename(&self, id: &str, name: String) -> ApiResult<()> {
        let _: Option<Version> = RecordIdFunc::from((Version::name(), id))
            .patch(&*DB, PatchOp::replace("/name", name))
            .await?;
        Ok(())
    }
    pub async fn update_translate_opts(&self, id: &str, translate_opts: String) -> ApiResult<()> {
        let _: Option<Version> = RecordIdFunc::from((Version::name(), id))
            .patch(&*DB, PatchOp::replace("/translate_opts", translate_opts))
            .await?;
        Ok(())
    }

    pub async fn list(&self, page: u32, limit: u32) -> ApiResult<Vec<RecordData<Version>>> {
        let offset = (page - 1) * limit;
        let search: Vec<RecordData<Version>> =
            Version::search(&*DB, Some(format!("LIMIT {limit} START {offset}"))).await?;
        Ok(search)
    }

    pub async fn get(&self, version: &str) -> ApiResult<RecordIdType<Version>> {
        let mut search: Vec<RecordData<Version>> =
            Version::search(&*DB, Some(format!("WHERE name = \"{version}\""))).await?;
        if search.is_empty() {
            return Ok(Version {
                name: version.to_owned(),
                translate_opts: None,
            }
            .add_i(&*DB)
            .await?
            .id
            .into());
        }
        Ok(search.remove(0).id.into())
    }
}
