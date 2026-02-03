use std::sync::Arc;

use actix_web::web::Data;
use apistos::web::{scope, Scope};
use manga_scraper::init::Services;

use crate::{
    models::{
        auth::AuthTokenDBService, chapter::ChapterDBService, kind::KindDBService,
        lists::ListDBService, logs::LogDbService, manga::MangaDBService, page::PageDBService,
        progress::UserProgressDBService, scraper::ScraperDbService, tag::TagDBService,
        user::UserDBService, version::VersionDBService, version_link::ChapterVersionDBService,
    },
    services::{auth::CryptoService, file::FileService},
};

use super::Config;

pub fn init_app_data(
    user: Data<UserDBService>,
    config: Arc<Config>,
    scrapers: Arc<Services>,
) -> Scope {
    scope("/api")
        .app_data(user)
        .app_data(Data::new(FileService::new(config.clone())))
        .app_data(Data::new(UserDBService::new()))
        .app_data(Data::new(ChapterDBService::default()))
        .app_data(Data::new(KindDBService::default()))
        .app_data(Data::new(ListDBService::default()))
        .app_data(Data::new(MangaDBService::default()))
        .app_data(Data::new(AuthTokenDBService::default()))
        .app_data(Data::new(PageDBService::default()))
        .app_data(Data::new(UserProgressDBService::default()))
        .app_data(Data::new(TagDBService::default()))
        .app_data(Data::new(VersionDBService::default()))
        .app_data(Data::new(ChapterVersionDBService::default()))
        .app_data(Data::new(LogDbService::default()))
        .app_data(Data::new(ScraperDbService::default()))
        .app_data(Data::new(CryptoService {
            claims: Default::default(),
            secret: config.secret_key.as_bytes().to_vec(),
        }))
        .app_data(Data::from(scrapers))
        .app_data(Data::from(config))
}
