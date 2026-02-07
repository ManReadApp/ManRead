use std::sync::Arc;
mod actions;
pub mod error;
mod init;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(init::env::get_env().expect("Failed  to load config"));
    init::logger::init_logger(&config.rust_log).expect("Failed to initialize config");
    let fs = storage::DiskStorage::new(&config.root_folder);
    let storage = storage::StorageSystem::new(&config.root_folder, Arc::new(fs))
        .await
        .expect("Failed to load storage system");
    let dbs = db::init_db(Default::default())
        .await
        .expect("Failed to load database");
    init::server::init_server(
        config.port,
        config.https_port,
        config,
        Arc::new(storage),
        dbs,
    )?
    .await
}
