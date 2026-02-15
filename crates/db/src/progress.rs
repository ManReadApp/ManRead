use serde::{Deserialize, Serialize};
use surrealdb::{opt::PatchOp, Datetime};
use surrealdb_extras::{RecordData, RecordIdFunc, RecordIdType, SurrealTable, SurrealTableInfo};

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
        let chapter_id = ChapterDBService::new(self.db.clone())
            .get_next_chapter(manga_id, chapter_id)
            .await?;
        self.update(user_id, manga_id, &chapter_id.id().to_string(), 0.0)
            .await?;
        Ok(())
    }
}
