use std::sync::Arc;

use actix_web::web::Data;
use apistos::web::{scope, Scope};
use db::DbHandle;
use storage::StorageSystem;

use crate::{
    actions::{
        auth::AuthAction, chapter::ChapterActions, chapter_version::ChapterVersionActions,
        crytpo::CryptoService, kind::KindActions, lists::ListActions, manga::MangaActions,
        reader::ReaderActions, tags::TagActions, token::TokenAction, user::UserActions,
    },
    init::env::Config,
};

pub fn init_app_data(config: Arc<Config>, fs: Arc<StorageSystem>, dbs: DbHandle) -> Scope {
    let crypto = Arc::new(CryptoService::new(config.secret_key.as_bytes().to_vec()));
    let auth = AuthAction {
        users: dbs.users.clone(),
        crypto: crypto.clone(),
        token: dbs.tokens.clone(),
        fs: fs.clone(),
    };
    let chapter = ChapterActions {
        chapters: dbs.chapters.clone(),
        tags: dbs.tags.clone(),
        versions: dbs.versions.clone(),
        mangas: dbs.mangas.clone(),
        pages: dbs.pages.clone(),
        fs: fs.clone(),
    };
    let cversion = ChapterVersionActions {
        versions: dbs.versions.clone(),
        chapters: dbs.chapters.clone(),
    };

    let kind = KindActions {
        kinds: dbs.kinds.clone(),
    };
    let lists = ListActions {
        mangas: dbs.mangas.clone(),
        lists: dbs.lists.clone(),
    };
    let manga = MangaActions {
        mangas: dbs.mangas.clone(),
        chapters: dbs.chapters.clone(),
        tags: dbs.tags.clone(),
        kinds: dbs.kinds.clone(),
        users: dbs.users.clone(),
        lists: dbs.lists.clone(),
        versions: dbs.versions,
        fs: fs.clone(),
    };

    let reader = ReaderActions {
        progresses: dbs.progress,
        chapters: dbs.chapters,
        pages: dbs.pages,
        chapter_versions: dbs.chapter_versions,
        mangas: dbs.mangas,
        lists: dbs.lists,
        kinds: dbs.kinds,
    };

    let tags = TagActions {
        tags: dbs.tags.clone(),
    };
    let token = TokenAction {
        token: dbs.tokens.clone(),
    };
    let user = UserActions {
        users: dbs.users,
        crypto,
        fs,
        tags: dbs.tags,
    };
    scope("/api")
        .app_data(Data::new(auth))
        .app_data(Data::new(chapter))
        .app_data(Data::new(cversion))
        .app_data(Data::new(kind))
        .app_data(Data::new(lists))
        .app_data(Data::new(manga))
        .app_data(Data::new(reader))
        .app_data(Data::new(tags))
        .app_data(Data::new(token))
        .app_data(Data::new(user))
        .app_data(Data::from(config))
}
