use api_structure::models::manga::tag::{Tag as GlobalTag, TagSex};
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surrealdb_extras::{RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable};
use surrealdb_extras::{SurrealTableInfo, ThingArray};

use crate::error::ApiResult;
use crate::init::db::DB;

#[derive(Default)]
pub struct TagDBService {}

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone, ApiComponent, JsonSchema)]
#[db("tags")]
#[sql(["DEFINE EVENT tag_updated ON TABLE tags WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Tag {
    /// Tag name
    pub tag: String,
    /// Description of the tag
    pub description: Option<String>,
    /// Target of the tag. E.g. Male, Female, None, Both,
    pub sex: u64,
}

#[derive(SurrealSelect, Deserialize)]
pub struct Empty {}

impl TagDBService {
    pub async fn search(&self, query: &str, page: u32, limit: u32) -> ApiResult<Vec<GlobalTag>> {
        let v: Vec<RecordData<Tag>> = Tag::search(
            &*DB,
            Some(format!("WHERE string::contains(tag, \"{}\")", query)),
        )
        .await?;
        Ok(v.into_iter()
            .map(|v| GlobalTag {
                tag: v.data.tag,
                description: v.data.description,
                sex: TagSex::from(v.data.sex),
            })
            .collect())
    }
    pub async fn get_tags_internal(&self, ids: Vec<RecordIdFunc>) -> ApiResult<Vec<Tag>> {
        let thing = ThingArray::from(ids);
        let items: Vec<Tag> = thing.get(&*DB).await?;
        Ok(items)
    }
    pub async fn get_tags(&self, ids: impl Iterator<Item = String>) -> ApiResult<Vec<GlobalTag>> {
        let thing = ThingArray::from(
            ids.map(|id| RecordIdFunc::from((Tag::name(), id.as_str())))
                .collect::<Vec<_>>(),
        );
        let items: Vec<Tag> = thing.get(&*DB).await?;

        Ok(items
            .into_iter()
            .map(|item| GlobalTag {
                tag: item.tag,
                description: item.description,
                sex: TagSex::from(item.sex),
            })
            .collect())
    }
    pub async fn get_ids(
        &self,
        tags: impl Iterator<Item = GlobalTag>,
    ) -> ApiResult<Vec<RecordIdType<Tag>>> {
        let mut out = vec![];
        for tag in tags {
            let search: Vec<RecordData<Empty>> = Tag::search(
                &*DB,
                Some(format!(
                    "WHERE tag = \"{}\" AND sex = {} LIMIT 1",
                    tag.tag, tag.sex
                )),
            )
            .await?;
            if let Some(v) = search.get(0) {
                out.push(v.id.clone());
            } else {
                let v = Tag {
                    tag: tag.tag,
                    description: tag.description,
                    sex: tag.sex as u64,
                }
                .add_i(&*DB)
                .await?;
                out.push(v.id);
            }
        }
        Ok(out.into_iter().map(Into::into).collect())
    }
}
