use std::{collections::HashSet, fmt::Display};

use scraper_module::ScrapedChapter;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};
use surrealdb_extras::{RecordData, RecordIdType, SurrealTable, SurrealTableInfo};

use crate::{
    chapter::ChapterDBService, error::DbResult, manga::Manga, tag::Empty, version::Version,
    DbSession,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("scraper_entry")]
pub struct ScraperEntry {
    pub manga: RecordIdType<Manga>,
    pub version: RecordIdType<Version>,
    pub state: State,
    pub data: ScrapedChapter,
}

#[derive(Deserialize)]
pub struct MangaVersion {
    pub manga: RecordIdType<Manga>,
    pub version: RecordIdType<Version>,
}

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("scraper")]
pub struct ScraperSearch {
    pub updated_at: Datetime,
}

#[derive(Clone)]
pub struct ScraperDbService {
    db: DbSession,
}

impl Default for ScraperDbService {
    fn default() -> Self {
        Self::new(crate::global_db())
    }
}

impl ScraperDbService {
    pub async fn next_chapter(&self) -> DbResult<Option<RecordData<ScraperEntry>>> {
        let v: Vec<RecordData<ScraperEntry>> = self
            .db
            .query(format!(
                "SELECT * FROM {} WHERE state == 'Approved' LIMIT 1",
                ScraperEntry::name()
            ))
            .await?
            .take(0)?;
        Ok(v.into_iter().next())
    }
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn get_items(
        &self,
        manga: RecordIdType<Manga>,
        version: RecordIdType<Version>,
    ) -> DbResult<Vec<RecordData<ScraperEntry>>> {
        let v: Vec<RecordData<ScraperEntry>> = self
            .db
            .query(format!(
                "SELECT * FROM {} WHERE manga = $manga AND version = $version",
                ScraperEntry::name()
            ))
            .bind(("manga", manga))
            .bind(("version", version))
            .await?
            .take(0)?;
        Ok(v)
    }

    pub async fn process_items(
        &self,
    ) -> DbResult<Vec<(RecordIdType<Manga>, RecordIdType<Version>)>> {
        let query = format!(
            "SELECT manga,version FROM {} WHERE state = 'Pending' GROUP by manga,version",
            ScraperEntry::name()
        );
        let data: Vec<MangaVersion> = self.db.query(query).await?.take(0)?;

        Ok(data.into_iter().map(|v| (v.manga, v.version)).collect())
    }

    pub async fn update(
        &self,
        manga: RecordIdType<Manga>,
        version: RecordIdType<Version>,
        chapters: Vec<ScrapedChapter>,
    ) -> DbResult<()> {
        //TODO: dont load all into memory

        let data: Vec<String> = self
            .db
            .query(format!(
                "(SELECT data.url FROM {} WHERE manga = $manga AND version = $version).data.url",
                ScraperEntry::name(),
            ))
            .bind(("manga", manga.clone()))
            .bind(("version", version.clone()))
            .await?
            .take(0)?;

        let urls = data.into_iter().collect::<HashSet<_>>();
        let items = chapters
            .into_iter()
            .filter(|v| !urls.contains(&v.url))
            .map(|v| ScraperEntry {
                manga: manga.clone(),
                version: version.clone(),
                state: State::default(),
                data: v,
            })
            .collect::<Vec<_>>();

        let mut new = vec![];
        for item in items {
            let exists = ChapterDBService::new(self.db.clone())
                .exists_by_url(item.data.url.clone())
                .await?;
            if !exists {
                new.push(item)
            }
        }

        if !new.is_empty() {
            let _: Vec<Empty> = self.db.insert(ScraperEntry::name()).content(new).await?;
        }

        let id = format!("{}###{}", manga.id().to_string(), version.id().to_string());

        let _: Option<Empty> = self
            .db
            .upsert((ScraperSearch::name(), id.as_str()))
            .content(ScraperSearch {
                updated_at: Datetime::default(),
            })
            .await?;
        Ok(())
    }

    pub async fn get_newer_then(
        &self,
        timestamp: Datetime,
    ) -> DbResult<Vec<(RecordIdType<Manga>, RecordIdType<Version>)>> {
        println!("loading");
        let data: Vec<RecordId> = self
            .db
            .query(format!(
                "(SELECT id FROM {} WHERE updated_at > $timestamp).id",
                ScraperSearch::name()
            ))
            .bind(("timestamp", timestamp))
            .await?
            .take(0)?;

        Ok(data
            .into_iter()
            .map(|v| {
                v.key()
                    .to_string()
                    .strip_prefix("⟨")
                    .unwrap()
                    .strip_suffix("⟩")
                    .unwrap()
                    .split_once("###")
                    .map(|v| {
                        (
                            RecordIdType::from((Manga::name(), v.0)),
                            RecordIdType::from((Version::name(), v.1)),
                        )
                    })
                    .unwrap()
            })
            .collect())
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum State {
    #[default]
    Pending,
    Declined,
    Approved,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Pending => write!(f, "Pending"),
            State::Declined => write!(f, "Declined"),
            State::Approved => write!(f, "Approved"),
        }
    }
}
