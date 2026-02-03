use std::{sync::Arc, time::Duration};

use manga_scraper::init::Services;
use surrealdb::Datetime;
use surrealdb_extras::{RecordData, RecordIdType};

use crate::{
    error::{ApiError, ApiResult, ErrorLogger},
    init::db::DB,
    models::{
        chapter::ChapterDBService,
        manga::{Manga, MangaDBService},
        page::PageDBService,
        scraper::{ScraperDbService, ScraperEntry},
    },
    services::{file::TempFile, BackgroundService},
    Config,
};

pub struct ScraperService {
    services: Arc<Services>,
    manga: MangaDBService,
    scraper: ScraperDbService,
}

pub struct ScraperService2 {
    services: Arc<Services>,
    chapter: ChapterDBService,
    pages: PageDBService,
    scraper: ScraperDbService,
    config: Arc<Config>,
}

#[async_trait::async_trait]
impl BackgroundService for ScraperService2 {
    async fn run(&self) {
        _ = self.process().await.map_err(|e| ErrorLogger::from(e));
    }

    fn when(&self) -> Duration {
        Duration::from_secs(60 * 30)
    }
}

impl ScraperService2 {
    pub fn new(config: Arc<Config>, services: Arc<Services>) -> Self {
        Self {
            services,
            chapter: ChapterDBService::default(),
            pages: PageDBService::default(),
            scraper: ScraperDbService::default(),
            config,
        }
    }
    pub async fn process(&self) -> ApiResult<()> {
        while let Some(item) = self.scraper.next_chapter().await? {
            _ = self.process_item(item).await.map_err(ErrorLogger::from);
        }
        Ok(())
    }

    async fn process_item(&self, item: RecordData<ScraperEntry>) -> ApiResult<()> {
        if let Some(scraper) = self
            .services
            .get(&item.data.data.url)
            .and_then(|v| v.reader.clone())
        {
            let chapter = match self
                .chapter
                .get(&item.data.manga.id().to_string(), item.data.data.chapter)
                .await
            {
                Ok(v) => v,
                Err(_) => self
                    .chapter
                    .create(
                        &item.data.manga.id().to_string(),
                        item.data.data.chapter,
                        vec![],
                        vec![],
                        vec![],
                        None,
                    )
                    .await?
                    .get(&*DB)
                    .await?
                    .ok_or(ApiError::NotFoundInDB)?,
            };
            // todo: check that version doesnt exist
            let mut created_pages = vec![];

            let pages = scraper.scrape_pages(&item.data.data.url).await?;
            for page in pages {
                let bytes = scraper.download_file(&page).await?;
                let format = match image::guess_format(&bytes) {
                    Ok(format) => format,
                    Err(_) => return Err(ApiError::CannotSaveTempFile),
                };
                let mut ext = format.extensions_str()[0];
                if ext == "jpg" {
                    ext = "jpeg";
                }
                created_pages.push(TempFile::from_bytes(&self.config.root_folder, bytes, ext));
            }

            let pages = self
                .pages
                .add(
                    &self.config.root_folder,
                    &item.data.manga.id().to_string(),
                    &chapter.id.id().to_string(),
                    &item.data.version.id().to_string(),
                    created_pages,
                )
                .await?;
            self.chapter
                .add(
                    &chapter.id.id().to_string(),
                    item.data.data.names,
                    vec![],
                    vec![item.data.data.url],
                    item.data.version,
                    pages,
                )
                .await?;
            item.id.delete_s(&*DB).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl BackgroundService for ScraperService {
    async fn run(&self) {
        _ = self.process().await.map_err(|e| ErrorLogger::from(e));
    }

    fn when(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }
}

impl ScraperService {
    pub fn new(services: Arc<Services>) -> Self {
        Self {
            services,
            manga: MangaDBService::default(),
            scraper: ScraperDbService::default(),
        }
    }

    pub async fn process(&self) -> ApiResult<()> {
        println!("downloading");
        let cancel = self
            .scraper
            .get_newer_then(Datetime::from(
                Datetime::default().into_inner().0 - Duration::from_secs(60 * 60 * 48),
            ))
            .await?;
        for item in self.manga.scrapers().await? {
            let item = item.get(&*DB).await?.ok_or(ApiError::NotFoundInDB)?;
            let record: RecordIdType<Manga> = RecordIdType::from(item.id);
            for scraper in item.data.scraper {
                if scraper.enabled && !cancel.contains(&(record.clone(), scraper.target.clone())) {
                    if let Some(v) = self
                        .services
                        .get(&scraper.url)
                        .and_then(|v| v.reader.clone())
                    {
                        if let Ok(chapters) = v.scrape_chapters(&scraper.url).await {
                            self.scraper
                                .update(record.clone(), scraper.target, chapters)
                                .await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
