use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{
    RecordData, RecordIdFunc, RecordIdType, SurrealSelect, SurrealTable, SurrealTableInfo,
};

use crate::{chapter::ChapterDBService, error::DbResult, tag::Empty, DbSession};

use super::{chapter::Chapter, manga::Manga, user::User};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("user_progress")]
#[sql(["DEFINE EVENT user_progress_updated ON TABLE user_progress WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct UserProgress {
    /// User who is reading
    pub user: RecordIdType<User>,
    /// Manga which is being read
    pub manga: RecordIdType<Manga>,
    /// Chapter which is being read
    pub chapter: RecordIdType<Chapter>,
    /// Progress of the chapter from 0.0 to 1.0
    pub progress: f64,
    #[opt(exclude = true)]
    pub updated: Datetime,
}

#[derive(Deserialize, SurrealSelect)]
struct ProgressUser {
    user: RecordIdType<User>,
}

#[derive(Clone)]
pub struct UserProgressDBService {
    db: DbSession,
}

impl UserProgressDBService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn get_progress(
        &self,
        user_id: &str,
        manga_id: &str,
    ) -> DbResult<(RecordIdType<Chapter>, f64)> {
        let mut record: Vec<RecordData<UserProgress>> = UserProgress::search(
            self.db.as_ref(),
            Some(format!(
                "WHERE user = {} AND manga = {} ORDER BY updated DESC LIMIT 1",
                RecordIdFunc::from((User::name(), user_id)),
                RecordIdFunc::from((Manga::name(), manga_id)),
            )),
        )
        .await?;
        if record.is_empty() {
            Err(crate::error::DbError::NotFound)
        } else {
            let ch = record.remove(0).data;
            Ok((ch.chapter, ch.progress))
        }
    }

    pub async fn update(
        &self,
        user_id: &str,
        manga_id: &str,
        chapter_id: &str,
        progress: f64,
    ) -> DbResult<()> {
        let mut record: Vec<RecordData<Empty>> = UserProgress::search(
            self.db.as_ref(),
            Some(format!(
                "WHERE user = {} AND manga = {} AND chapter = {} LIMIT 1",
                RecordIdFunc::from((User::name(), user_id)),
                RecordIdFunc::from((Manga::name(), manga_id)),
                RecordIdFunc::from((Chapter::name(), chapter_id)),
            )),
        )
        .await?;
        if record.is_empty() {
            UserProgress {
                user: RecordIdType::from((User::name(), user_id)),
                manga: RecordIdType::from((Manga::name(), manga_id)),
                chapter: RecordIdType::from((Chapter::name(), chapter_id)),
                progress,
                updated: Default::default(),
            }
            .add(self.db.as_ref())
            .await?;
        } else {
            let _: Option<Empty> = record
                .remove(0)
                .id
                .patch(self.db.as_ref(), PatchOp::replace("/progress", progress))
                .await?;
        }
        Ok(())
    }

    pub async fn load_next_chapter(
        &self,
        user_id: &str,
        manga_id: &str,
        chapter_id: &str,
    ) -> DbResult<()> {
        if let Some(chapter_id) = ChapterDBService::new(self.db.clone())
            .get_next_chapter(manga_id, chapter_id)
            .await
            .ok()
        {
            self.update(user_id, manga_id, &chapter_id.id().to_string(), 0.0)
                .await?;
        }

        Ok(())
    }

    pub async fn recompute_for_new_chapter(
        &self,
        manga_id: &str,
        new_chapter_id: &str,
        new_chapter_number: f64,
    ) -> DbResult<()> {
        let manga_ref = RecordIdFunc::from((Manga::name(), manga_id));
        let query = format!(
            "SELECT id, chapter FROM {} WHERE id IN (SELECT chapters FROM {})[0].chapters AND chapter < $chapter ORDER BY chapter DESC LIMIT 1",
            Chapter::name(),
            manga_ref
        );
        let mut previous: Vec<RecordData<Empty>> = self
            .db
            .query(query)
            .bind(("chapter", new_chapter_number))
            .await?
            .take(0)?;
        if previous.is_empty() {
            return Ok(());
        }
        let previous_chapter = previous.remove(0).id;
        let new_chapter_ref = RecordIdFunc::from((Chapter::name(), new_chapter_id)).to_string();
        let manga_ref = RecordIdFunc::from((Manga::name(), manga_id)).to_string();
        let completed: Vec<RecordData<ProgressUser>> = UserProgress::search(
            self.db.as_ref(),
            Some(format!(
                "WHERE manga = {manga_ref} AND chapter = {} AND progress >= 0.95",
                previous_chapter
            )),
        )
        .await?;

        for progress in completed {
            let user_ref = RecordIdFunc::from(progress.data.user.clone()).to_string();
            let existing: Vec<RecordData<Empty>> = UserProgress::search(
                self.db.as_ref(),
                Some(format!(
                    "WHERE user = {user_ref} AND manga = {manga_ref} AND chapter = {new_chapter_ref} LIMIT 1"
                )),
            )
            .await?;
            if existing.is_empty() {
                UserProgress {
                    user: progress.data.user,
                    manga: RecordIdType::from((Manga::name(), manga_id)),
                    chapter: RecordIdType::from((Chapter::name(), new_chapter_id)),
                    progress: 0.0,
                    updated: Default::default(),
                }
                .add(self.db.as_ref())
                .await?;
            }
        }
        Ok(())
    }
}
