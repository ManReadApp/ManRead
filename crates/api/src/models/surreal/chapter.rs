use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo as _,
    ThingArray,
};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
};

use super::{
    manga::Manga,
    page::Page,
    tag::{Empty, Tag},
    version::Version,
    version_link::ChapterVersion,
};

#[derive(Deserialize, SurrealSelect)]
pub struct ChapterPart {
    pub titles: Vec<String>,
    pub chapter: f64,
    pub tags: Vec<RecordIdType<Tag>>,
    pub sources: Vec<String>,
    pub release_date: Option<Datetime>,
}

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("chapters")]
#[sql(["DEFINE EVENT chapter_updated ON TABLE chapters WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Chapter {
    /// Titles. like chapter 1.5, chapter 1.5 - NAME
    pub titles: Vec<String>,
    /// Ordering/chapter number.
    pub chapter: f64,
    /// tags of a chapter that will be in generated_tags
    pub tags: Vec<RecordIdType<Tag>>,
    /// Origin of files & trackers
    pub sources: Vec<String>,
    /// When a chapter was published
    pub release_date: Option<Datetime>,
    ///map of Version to ChapterVersion which is needed to get files
    pub versions: HashMap<String, RecordIdType<ChapterVersion>>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(Serialize, Deserialize)]
pub struct ChapterTag {
    pub tags: Vec<RecordIdType<Tag>>,
}
#[derive(Default)]
pub struct ChapterDBService;

#[derive(Deserialize)]
pub struct ChapterPart2 {
    chapter: f64,
    titles: Vec<String>,
    versions: Vec<String>,
}

impl ChapterDBService {
    pub async fn get_chapter_by_version(
        &self,
        manga_id: RecordIdType<Manga>,
        version: RecordIdType<Version>,
    ) -> ApiResult<Vec<(RecordIdFunc, f64, Vec<String>)>> {
        println!("loaded");
        let version = version.to_string();
        let query:Vec<RecordData<ChapterPart2>> = DB.query(format!("SELECT id, chapter,titles,object::keys(versions) as versions FROM (SELECT chapters FROM {})[0].chapters;", manga_id)).await?.take(0)?;
        Ok(query
            .into_iter()
            .filter(|v| v.data.versions.contains(&version))
            .map(|v| (v.id, v.data.chapter, v.data.titles))
            .collect::<Vec<_>>())
    }

    pub async fn exists_by_url(url: String) -> ApiResult<bool> {
        let count: Option<usize> = DB
            .query(format!(
                "(SELECT count() FROM {} WHERE sources CONTAINS $url LIMIT 1)[0].count",
                Chapter::name()
            ))
            .bind(("url", url))
            .await?
            .take(0)?;

        Ok(count.unwrap_or_default() > 0)
    }
    pub async fn get_tags(
        &self,
        chaptrs: Vec<RecordIdType<Chapter>>,
    ) -> ApiResult<Vec<RecordIdType<Tag>>> {
        let v = ThingArray(chaptrs.into_iter().map(|v| v.thing.0).collect());
        let v: Vec<ChapterTag> = v.get(&*DB).await?;
        let v = v.into_iter().flat_map(|v| v.tags).collect();
        Ok(v)
    }
    pub async fn get_next_chapter(
        &self,
        manga_id: &str,
        chapter_id: &str,
    ) -> ApiResult<RecordIdType<Chapter>> {
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let chapter_id = RecordIdFunc::from((Chapter::name(), chapter_id)).to_string();
        let query = format!("SELECT id, chapter FROM (SELECT chapters FROM {})[0].chapters WHERE chapter > (SELECT chapter FROM {})[0].chapter ORDER BY chapter LIMIT 1;", manga_id, chapter_id);
        let mut a: Vec<RecordData<Empty>> = DB.query(query).await?.take(0)?;
        if a.is_empty() {
            Err(ApiError::NotFoundInDB)
        } else {
            Ok(a.remove(0).id.into())
        }
    }

    pub async fn get_manga_id(&self, chapter_id: &str) -> ApiResult<String> {
        let chapter_id = RecordIdFunc::from((Chapter::name(), chapter_id)).to_string();
        let mut v: Vec<RecordData<Empty>> =
            Manga::search(&*DB, Some(format!("WHERE {chapter_id} in chapters"))).await?;
        if v.is_empty() {
            return Err(ApiError::NotFoundInDB);
        }
        Ok(v.remove(0).id.id().to_string().to_owned())
    }

    pub async fn delete(&self, chapter_id: &str) -> ApiResult<()> {
        todo!()
    }
    pub async fn delete_version(&self, chapter_id: &str, version_id: &str) -> ApiResult<String> {
        todo!()
    }

    pub async fn get_by_id(&self, chapter_id: &str) -> ApiResult<Chapter> {
        RecordIdFunc::from((Chapter::name(), chapter_id))
            .get(&*DB)
            .await?
            .ok_or(ApiError::NotFoundInDB)
    }

    pub async fn create(
        &self,
        manga_id: &str,
        chapter: f64,
        titles: Vec<String>,
        tags: Vec<RecordIdType<Tag>>,
        sources: Vec<String>,
        release_date: Option<Datetime>,
    ) -> ApiResult<RecordIdType<Chapter>> {
        let id = Chapter {
            titles,
            chapter,
            tags,
            sources,
            release_date,
            versions: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        }
        .add(&*DB)
        .await?
        .ok_or(ApiError::NotFoundInDB)?;
        //TODO: add tags to manga generated
        let _: Option<Empty> = RecordIdFunc::from((Manga::name(), manga_id))
            .patch(&*DB, PatchOp::add("/chapters", id.id.clone()))
            .await?;

        //TODO: also reload chapter progress
        Ok(id.id.into())
    }
    pub async fn add(
        &self,
        chapter_id: &str,
        titles: Vec<String>,
        tags: Vec<RecordIdType<Tag>>,
        sources: Vec<String>,
        version: RecordIdType<Version>,
        pages: Vec<RecordIdType<Page>>,
    ) -> ApiResult<()> {
        let value = ChapterVersion {
            version: version.clone(),
            pages,
            link: None,
            updated: Default::default(),
            created: Default::default(),
        }
        .add_i(&*DB)
        .await?
        .id;
        let ch = RecordIdFunc::from((Chapter::name(), chapter_id));
        let _: Option<Empty> = ch
            .clone()
            .patch(
                &*DB,
                PatchOp::add(&format!("/versions/{}", version.to_string()), value),
            )
            .await?;
        for title in titles {
            let _: Option<Empty> = ch
                .clone()
                .patch(&*DB, PatchOp::add("/titles", title))
                .await?;
        }
        for tag in tags {
            let _: Option<Empty> = ch.clone().patch(&*DB, PatchOp::add("/tags", tag)).await?;
        }
        for source in sources {
            let _: Option<Empty> = ch
                .clone()
                .patch(&*DB, PatchOp::add("/sources", source))
                .await?;
        }
        Ok(())
    }

    pub async fn get(&self, manga_id: &str, chapter: f64) -> ApiResult<RecordData<Chapter>> {
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let query = format!(
            "SELECT * FROM {} WHERE id IN (SELECT foreign_ids FROM {}) AND chapter = {chapter} LIMIT 1;",
            Chapter::name(),
            manga_id,
        );
        let mut res: Vec<RecordData<Chapter>> = DB.query(query).await?.take(0)?;
        if res.is_empty() {
            return Err(ApiError::NotFoundInDB);
        }
        Ok(res.remove(0))
    }

    pub async fn get__detail(
        &self,
        ids: impl Iterator<Item = RecordIdType<Chapter>>,
    ) -> ApiResult<Vec<RecordData<Chapter>>> {
        let v: Vec<RecordData<Chapter>> =
            ThingArray::from(ids.collect::<Vec<_>>()).get(&*DB).await?;
        Ok(v)
    }

    pub async fn get_simple(
        &self,
        ids: impl Iterator<Item = RecordIdType<Chapter>>,
    ) -> ApiResult<Vec<RecordData<ChapterPart>>> {
        let v: Vec<RecordData<ChapterPart>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get_part(&*DB)
            .await?;
        Ok(v)
    }
}
