use std::sync::Arc;
mod actions;
pub mod error;
mod init;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(init::env::get_env()?);
    init::logger::init_logger(&config.rust_log)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let fs = storage::DiskStorage::new(&config.root_folder);
    let storage = storage::StorageSystem::new(&config.root_folder, Arc::new(fs))
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let dbs = db::init_db(Default::default())
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    init::server::init_server(
        config.port,
        config.https_port,
        config,
        Arc::new(storage),
        dbs,
    )?
    .await
}
