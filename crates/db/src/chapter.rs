use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo as _,
    ThingArray,
};

use crate::{
    error::{DbError, DbResult},
    DbSession,
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
#[derive(Clone)]
pub struct ChapterDBService {
    db: DbSession,
}

#[derive(Deserialize)]
pub struct ChapterPart2 {
    chapter: f64,
    titles: Vec<String>,
    versions: Vec<String>,
}

impl ChapterDBService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn get_chapter_by_version(
        &self,
        manga_id: RecordIdType<Manga>,
        version: RecordIdType<Version>,
    ) -> DbResult<Vec<(RecordIdFunc, f64, Vec<String>)>> {
        println!("loaded");
        let version = version.to_string();
        let query:Vec<RecordData<ChapterPart2>> = self.db.query(format!("SELECT id, chapter,titles,object::keys(versions) as versions FROM (SELECT chapters FROM {})[0].chapters;", manga_id)).await?.take(0)?;
        Ok(query
            .into_iter()
            .filter(|v| v.data.versions.contains(&version))
            .map(|v| (v.id, v.data.chapter, v.data.titles))
            .collect::<Vec<_>>())
    }

    pub async fn exists_by_url(&self, url: String) -> DbResult<bool> {
        let count: Option<usize> = self
            .db
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
    ) -> DbResult<Vec<RecordIdType<Tag>>> {
        let v = ThingArray(chaptrs.into_iter().map(|v| v.thing.0).collect());
        let v: Vec<ChapterTag> = v.get(self.db.as_ref()).await?;
        let v = v.into_iter().flat_map(|v| v.tags).collect();
        Ok(v)
    }
    pub async fn get_next_chapter(
        &self,
        manga_id: &str,
        chapter_id: &str,
    ) -> DbResult<RecordIdType<Chapter>> {
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let chapter_id = RecordIdFunc::from((Chapter::name(), chapter_id)).to_string();
        let query = format!("SELECT id, chapter FROM (SELECT chapters FROM {})[0].chapters WHERE chapter > (SELECT chapter FROM {})[0].chapter ORDER BY chapter LIMIT 1;", manga_id, chapter_id);
        let mut a: Vec<RecordData<Empty>> = self.db.query(query).await?.take(0)?;
        if a.is_empty() {
            Err(DbError::NotFound)
        } else {
            Ok(a.remove(0).id.into())
        }
    }

    pub async fn get_manga_id(&self, chapter_id: &str) -> DbResult<String> {
        let chapter_id = RecordIdFunc::from((Chapter::name(), chapter_id)).to_string();
        let mut v: Vec<RecordData<Empty>> = Manga::search(
            self.db.as_ref(),
            Some(format!("WHERE {chapter_id} in chapters")),
        )
        .await?;
        if v.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(v.remove(0).id.id().to_string().to_owned())
    }

    pub async fn delete(&self, chapter_id: &str) -> DbResult<()> {
        let chapter = self.get_by_id(chapter_id).await?;
        let manga_id = self.get_manga_id(chapter_id).await?;
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id.as_str()));

        let mut manga: Manga = manga_id
            .clone()
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;

        manga
            .chapters
            .retain(|v| v.id().to_string().as_str() != chapter_id);

        let _: Option<RecordData<Empty>> = manga_id
            .patch(
                self.db.as_ref(),
                PatchOp::replace("/chapters", manga.chapters),
            )
            .await?;

        let chapter_id = RecordIdFunc::from((Chapter::name(), chapter_id));
        chapter_id.delete_s(self.db.as_ref()).await?;

        for version in chapter.versions.into_values() {
            RecordIdFunc::from(version)
                .delete_s(self.db.as_ref())
                .await?;
        }

        Ok(())
    }
    pub async fn delete_version(&self, chapter_id: &str, version_id: &str) -> DbResult<String> {
        let chapter_id_ref = RecordIdFunc::from((Chapter::name(), chapter_id));
        let mut chapter: Chapter = chapter_id_ref
            .clone()
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;

        let version_key = RecordIdFunc::from((Version::name(), version_id)).to_string();
        let removed = chapter
            .versions
            .remove(&version_key)
            .ok_or(DbError::NotFound)?;

        let _: Option<RecordData<Empty>> = chapter_id_ref
            .patch(
                self.db.as_ref(),
                PatchOp::replace("/versions", chapter.versions),
            )
            .await?;

        RecordIdFunc::from(removed.clone())
            .delete_s(self.db.as_ref())
            .await?;

        Ok(removed.id().to_string())
    }

    pub async fn get_by_id(&self, chapter_id: &str) -> DbResult<Chapter> {
        RecordIdFunc::from((Chapter::name(), chapter_id))
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)
    }

    pub async fn edit(
        &self,
        chapter_id: &str,
        titles: Option<Vec<String>>,
        chapter: Option<f64>,
        tags: Option<Vec<RecordIdType<Tag>>>,
        sources: Option<Vec<String>>,
        release_date: Option<Option<Datetime>>,
    ) -> DbResult<()> {
        //TODO: delete the files and pages
        let id = RecordIdFunc::from((Chapter::name(), chapter_id));
        let _: Option<RecordData<Empty>> = id
            .clone()
            .get(self.db.as_ref())
            .await?
            .ok_or(DbError::NotFound)?;

        if let Some(titles) = titles {
            let _: Option<RecordData<Empty>> = id
                .clone()
                .patch(self.db.as_ref(), PatchOp::replace("/titles", titles))
                .await?;
        }

        if let Some(chapter) = chapter {
            let _: Option<RecordData<Empty>> = id
                .clone()
                .patch(self.db.as_ref(), PatchOp::replace("/chapter", chapter))
                .await?;
        }

        if let Some(tags) = tags {
            let _: Option<RecordData<Empty>> = id
                .clone()
                .patch(self.db.as_ref(), PatchOp::replace("/tags", tags))
                .await?;
        }

        if let Some(sources) = sources {
            let _: Option<RecordData<Empty>> = id
                .clone()
                .patch(self.db.as_ref(), PatchOp::replace("/sources", sources))
                .await?;
        }

        if let Some(release_date) = release_date {
            let _: Option<RecordData<Empty>> = id
                .patch(
                    self.db.as_ref(),
                    PatchOp::replace("/release_date", release_date),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn create(
        &self,
        manga_id: &str,
        chapter: f64,
        titles: Vec<String>,
        tags: Vec<RecordIdType<Tag>>,
        sources: Vec<String>,
        release_date: Option<Datetime>,
    ) -> DbResult<RecordIdType<Chapter>> {
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
        .add(self.db.as_ref())
        .await?
        .ok_or(DbError::NotFound)?;
        //TODO: add tags to manga generated
        let _: Option<Empty> = RecordIdFunc::from((Manga::name(), manga_id))
            .patch(self.db.as_ref(), PatchOp::add("/chapters", id.id.clone()))
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
    ) -> DbResult<()> {
        let value = ChapterVersion {
            version: version.clone(),
            pages,
            link: None,
            updated: Default::default(),
            created: Default::default(),
        }
        .add_i(self.db.as_ref())
        .await?
        .id;
        let ch = RecordIdFunc::from((Chapter::name(), chapter_id));
        let _: Option<Empty> = ch
            .clone()
            .patch(
                self.db.as_ref(),
                PatchOp::add(&format!("/versions/{}", version.to_string()), value),
            )
            .await?;
        for title in titles {
            let _: Option<Empty> = ch
                .clone()
                .patch(self.db.as_ref(), PatchOp::add("/titles", title))
                .await?;
        }
        for tag in tags {
            let _: Option<Empty> = ch
                .clone()
                .patch(self.db.as_ref(), PatchOp::add("/tags", tag))
                .await?;
        }
        for source in sources {
            let _: Option<Empty> = ch
                .clone()
                .patch(self.db.as_ref(), PatchOp::add("/sources", source))
                .await?;
        }
        Ok(())
    }

    pub async fn get(&self, manga_id: &str, chapter: f64) -> DbResult<RecordData<Chapter>> {
        let manga_id = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let query = format!(
            "SELECT * FROM {} WHERE id IN (SELECT foreign_ids FROM {}) AND chapter = {chapter} LIMIT 1;",
            Chapter::name(),
            manga_id,
        );
        let mut res: Vec<RecordData<Chapter>> = self.db.query(query).await?.take(0)?;
        if res.is_empty() {
            return Err(DbError::NotFound);
        }
        Ok(res.remove(0))
    }

    pub async fn get_detail(
        &self,
        ids: impl Iterator<Item = RecordIdType<Chapter>>,
    ) -> DbResult<Vec<RecordData<Chapter>>> {
        let v: Vec<RecordData<Chapter>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get(self.db.as_ref())
            .await?;
        Ok(v)
    }

    pub async fn get_simple(
        &self,
        ids: impl Iterator<Item = RecordIdType<Chapter>>,
    ) -> DbResult<Vec<RecordData<ChapterPart>>> {
        let v: Vec<RecordData<ChapterPart>> = ThingArray::from(ids.collect::<Vec<_>>())
            .get_part(self.db.as_ref())
            .await?;
        Ok(v)
    }
}
