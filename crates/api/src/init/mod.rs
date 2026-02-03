mod app_data;
mod background;
mod cors;
mod data;
pub mod db;
mod env;
#[cfg(feature = "https")]
mod https;
mod logger;
mod server;
use std::sync::Arc;

use crate::models::user::UserDBService;
use actix_web::web::Data;
pub use env::random_string;
pub use env::Config;
#[cfg(not(feature = "https"))]
pub type HttpsBuilderType = ();
#[cfg(feature = "https")]
pub type HttpsBuilderType = openssl::ssl::SslAcceptorBuilder;

pub async fn init() -> std::io::Result<()> {
    let config = Arc::new(env::get_env().expect("Failed  to load config"));
    logger::init_logger(&config.rust_log).expect("Failed to initialize config");
    data::init_data(&config.root_folder).expect("Failed to initialize data folder");
    db::init_db().await.expect("Failed to initalize db");
    #[cfg(not(feature = "https"))]
    let https_builder = ();
    #[cfg(feature = "https")]
    let https_builder = https::init_https(&config.root_folder);
    // Insert new setup here
    let user = Data::new(UserDBService::new());
    let u = user.clone();
    let scrapers = Arc::new(
        manga_scraper::init::register(&config.root_folder).expect("Failed to load scrapers"),
    );
    let scrapers2 = scrapers.clone();
    let c = config.clone();
    tokio::spawn(async move {
        // background::BackgroundServiceRunner::new(u, c, scrapers2)
        //     .run()
        //     .await;
    });
    let c = config.clone();
    let app_data = move || app_data::init_app_data(user.clone(), c.clone(), scrapers.clone());
    logger::log_url(&config);
    server::init_server(
        config.port,
        config.https_port,
        https_builder,
        config,
        app_data,
    )?
    .await?;
    Ok(())
}
