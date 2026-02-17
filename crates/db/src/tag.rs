use api_structure::v1::{Tag as GlobalTag, TagSex};
use serde::{Deserialize, Serialize};
use surrealdb_extras::{RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable};
use surrealdb_extras::{SurrealTableInfo, ThingArray};

use crate::error::DbResult;
use crate::DbSession;

#[derive(Clone)]
pub struct TagDBService {
    db: DbSession,
}

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
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
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn search(&self, query: &str) -> DbResult<Vec<GlobalTag>> {
        let v: Vec<RecordData<Tag>> = Tag::search(
            self.db.as_ref(),
            Some(format!("WHERE string::contains(tag, \"{}\")", query)),
        )
        .await?;
        Ok(v.into_iter()
            .map(|v| GlobalTag {
                tag: v.data.tag,
                description: v.data.description,
                sex: TagSex::try_from(v.data.sex).unwrap_or(TagSex::Unknown),
            })
            .collect())
    }
    pub async fn get_tags_internal(&self, ids: Vec<RecordIdFunc>) -> DbResult<Vec<Tag>> {
        let thing = ThingArray::from(ids);
        let items: Vec<Tag> = thing.get(self.db.as_ref()).await?;
        Ok(items)
    }
    pub async fn get_tags(&self, ids: impl Iterator<Item = String>) -> DbResult<Vec<GlobalTag>> {
        let thing = ThingArray::from(
            ids.map(|id| RecordIdFunc::from((Tag::name(), id.as_str())))
                .collect::<Vec<_>>(),
        );
        let items: Vec<Tag> = thing.get(self.db.as_ref()).await?;

        Ok(items
            .into_iter()
            .map(|item| GlobalTag {
                tag: item.tag,
                description: item.description,
                sex: TagSex::try_from(item.sex).unwrap_or(TagSex::Unknown),
            })
            .collect())
    }
    pub async fn get_ids(
        &self,
        tags: impl Iterator<Item = GlobalTag>,
    ) -> DbResult<Vec<RecordIdType<Tag>>> {
        let mut out = vec![];
        for tag in tags {
            let mut result = self
                .db
                .query("SELECT id FROM tags WHERE tag = $tag AND sex = $sex LIMIT 1")
                .bind(("tag", tag.tag.clone()))
                .bind(("sex", tag.sex as u64))
                .await?;
            let search: Vec<RecordData<Empty>> = result.take(0)?;
            if let Some(v) = search.get(0) {
                out.push(v.id.clone());
            } else {
                let v = Tag {
                    tag: tag.tag,
                    description: tag.description,
                    sex: tag.sex as u64,
                }
                .add_i(self.db.as_ref())
                .await?;
                out.push(v.id);
            }
        }
        Ok(out.into_iter().map(Into::into).collect())
    }
}
