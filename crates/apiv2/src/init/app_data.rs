use std::sync::Arc;

use actix_web::web::Data;
use apistos::web::{scope, Scope};
use db::DbHandle;
use storage::StorageSystem;

use crate::{
    actions::{auth::AuthAction, crytpo::CryptoService, token::TokenAction},
    init::env::Config,
};

pub fn init_app_data(config: Arc<Config>, fs: Arc<StorageSystem>, dbs: DbHandle) -> Scope {
    let crypto = Arc::new(CryptoService::new(config.secret_key.as_bytes().to_vec()));
    let auth = AuthAction {
        users: dbs.users,
        crypto: crypto.clone(),
        token: dbs.tokens.clone(),
        fs,
    };
    let token = TokenAction {
        token: dbs.tokens.clone(),
    };
    scope("/api")
        .app_data(Data::new(auth))
        .app_data(Data::new(token))
        .app_data(Data::from(config))
}
