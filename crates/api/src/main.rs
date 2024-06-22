use crate::env::config::Config;
use crate::errors::ApiResult;
use crate::services::auth_service::validator;
use crate::services::crypto_service::CryptoService;
use crate::services::db::auth_tokens::AuthTokenDBService;
use crate::services::db::chapter::ChapterDBService;
use crate::services::db::chapter_version::ChapterVersionDBService;
use crate::services::db::establish;
use crate::services::db::manga::MangaDBService;
use crate::services::db::manga_kind::MangaKindDBService;
use crate::services::db::manga_list::MangaListDBService;
use crate::services::db::page::PageDBService;
use crate::services::db::progress::ProgressDBService;
use crate::services::db::scrape_account::ScrapeAccountDBService;
use crate::services::db::scrape_list::ScrapeListDBService;
use crate::services::db::tag::TagDBService;
use crate::services::db::user::UserDBService;
use crate::services::db::version::VersionDBService;
use crate::services::uri_service::UriService;
use crate::util::create_folders;
use actix_files::NamedFile;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{post, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::fonts::FontRequest;
use fern::colors::{Color, ColoredLevelConfig};
use log::{info, LevelFilter};
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

mod env;
mod errors;
mod routes;
mod services;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = env::config::get_env().expect("Server configuration failed");
    create_folders(
        &config.root_folder,
        vec![
            "frontend",
            "covers",
            "mangas",
            "ssl",
            "temp",
            "external",
            "users/banner",
            "users/icon",
        ],
    )?;
    #[cfg(feature = "dev")]
    let _ = std::os::unix::fs::symlink(
        std::fs::canonicalize(&config.root_folder).unwrap(),
        PathBuf::from("crates/scraper/tests"),
    );
    let colors = ColoredLevelConfig::default()
        .trace(Color::Cyan)
        .debug(Color::Blue)
        .info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::from_str(&config.rust_log).unwrap())
        .level_for("selectors", LevelFilter::Off) //remove logger for scraping
        .level_for("html5ever", LevelFilter::Off)
        .level_for("hyper", LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    let db = Arc::new(establish(config.root_folder.clone(), true).await.unwrap());
    log_url(&config);
    #[cfg(feature = "https")]
    let ssl_builder = {
        let mut builder =
            openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())
                .expect("Couldnt initialize SslAcceptor");
        builder
            .set_private_key_file(
                config.root_folder.join("ssl/key.pem"),
                openssl::ssl::SslFiletype::PEM,
            )
            .expect("File does not exist");
        builder
            .set_certificate_chain_file(config.root_folder.join("ssl/cert.pem"))
            .expect("File does not exist");
        builder
    };
    let dbc = db.clone();
    let cfgc = config.clone();
    let hs = HttpServer::new(move || {
        let logger = Logger::default();
        let app = App::new().wrap(logger);
        let external =
            UriService::new(manread_scraper::ExternalSite::init(cfgc.root_folder.clone()).unwrap());
        let (multi, single, search, meta) =
            manread_scraper::init(cfgc.root_folder.clone()).unwrap();
        #[cfg(all(feature = "cors", not(feature = "cors-permissive")))]
        let app = app.wrap(
            actix_cors::Cors::default()
                .allow_any_header()
                .allowed_methods(vec!["GET", "POST"])
                .supports_credentials()
                .max_age(3600),
        );
        #[cfg(all(feature = "cors", feature = "cors-permissive"))]
        let app = app.wrap(actix_cors::Cors::permissive());
        let app = app
            .app_data(Data::from(dbc.clone()))
            .app_data(Data::new(CryptoService {
                secret: cfgc.secret_key.as_bytes().to_vec(),
            }))
            .app_data(Data::new(cfgc.clone()))
            .app_data(Data::new(fonts()))
            .app_data(Data::new(AuthTokenDBService::new(dbc.clone())))
            .app_data(Data::new(ChapterDBService::new(dbc.clone())))
            .app_data(Data::new(ChapterVersionDBService::new(dbc.clone())))
            .app_data(Data::new(MangaDBService::new(dbc.clone())))
            .app_data(Data::new(MangaKindDBService::new(dbc.clone())))
            .app_data(Data::new(MangaListDBService::new(dbc.clone())))
            .app_data(Data::new(PageDBService::new(dbc.clone())))
            .app_data(Data::new(ProgressDBService::new(dbc.clone())))
            .app_data(Data::new(ScrapeAccountDBService::new(dbc.clone())))
            .app_data(Data::new(ScrapeListDBService::new(dbc.clone())))
            .app_data(Data::new(TagDBService::new(dbc.clone())))
            .app_data(Data::new(UserDBService::new(dbc.clone())))
            .app_data(Data::new(VersionDBService::new(dbc.clone())))
            .app_data(Data::new(external))
            .app_data(Data::new(search))
            .app_data(Data::new(single))
            .app_data(Data::new(multi))
            .app_data(Data::new(meta))
            .service(web::redirect(
                "/source",
                "https://github.com/ManReadApp/ManRead",
            ))
            .service(web::redirect(
                "/github",
                "https://github.com/ManReadApp/ManRead",
            ))
            .service(web::redirect("/discord", "https://discord.gg/FeEe4rDS"))
            .service(web::redirect("/ko_fi", "https://ko-fi.com/manread"))
            .service(web::redirect("/kofi", "https://ko-fi.com/manread"))
            .service(web::redirect("/ko-fi", "https://ko-fi.com/manread"))
            .service(web::redirect("/sponsor", "https://ko-fi.com/manread"))
            .service(web::redirect("/donate", "https://ko-fi.com/manread"))
            .service(web::redirect("/tip", "https://ko-fi.com/manread"))
            .service(routes::frontend::frontend_empty_ep)
            .service(routes::frontend::frontend_ep)
            .service(
                web::scope("/api")
                    .service(get_fonts)
                    .service(get_font)
                    .service(routes::image::upload_images)
                    .service(routes::image::spinner)
                    .service(routes::user::sign_up_route)
                    .service(routes::user::sign_in_route)
                    .service(routes::user::reset_password_route)
                    .service(routes::user::request_reset_password_route)
                    .service(
                        web::scope("")
                            .wrap(HttpAuthentication::bearer(validator))
                            .service(
                                web::scope("/manga")
                                    .service(routes::manga::get_tags_route)
                                    .service(routes::manga::get_kinds_route),
                            )
                            .service(routes::user::refresh_route) //ALL
                            .service(routes::user::activate_route) //NotVerified
                            .service(routes::manga::home_route) //min User
                            .service(routes::manga::search_route) //min User
                            .service(routes::manga::cover_route) //min User
                            .service(routes::manga::info_route) //min User
                            .service(routes::manga::reader_info_route) //min User
                            .service(routes::manga::pages_route) //min User
                            .service(routes::manga::chapter_page_route) //min User
                            .service(routes::manga::translation_route) //min User
                            .service(routes::manga::external_search) //min User
                            .service(routes::manga::available_external_search_sites), //min User
                    ),
            );
        app
    })
    .bind(format!("0.0.0.0:{}", config.port))?;

    #[cfg(feature = "https")]
    let hs = hs.bind_openssl(format!("0.0.0.0:{}", config.https_port), ssl_builder)?;
    let (res, _) = tokio::join!(hs.run(), test(|| { db.clone() }));
    res
}

//TODO: wtf
async fn test(_: impl Fn() -> Arc<Surreal<Db>>) {}

fn log_url(config: &Config) {
    #[cfg(feature = "log-ip")]
    if let Ok(ip) = local_ip_address::local_ip() {
        info!("Starting server at http://{}:{}/", &ip, config.port);
    }
    info!("Starting server at http://0.0.0.0:{}/", config.port);
}

pub type Fonts = HashMap<String, PathBuf>;

fn fonts() -> Fonts {
    let mut archive = HashMap::new();
    for file in read_dir("data/fonts").unwrap() {
        let file = file.unwrap();
        let name = file.file_name().to_str().unwrap_or_default().to_lowercase();
        if name.ends_with(".ttf") || name.ends_with(".otf") {
            archive.insert(
                file.file_name().to_str().unwrap_or_default()[..name.len() - 4].to_string(),
                file.path(),
            );
        }
    }
    archive
}

#[post("/fonts")]
pub async fn get_fonts(data: Data<Fonts>) -> Json<Vec<String>> {
    Json(data.iter().map(|(v, _)| v.to_string()).collect())
}
#[post("/font")]
pub async fn get_font(Json(request): Json<FontRequest>, data: Data<Fonts>) -> ApiResult<NamedFile> {
    let path = data.get(&request.file).ok_or(ApiErr {
        message: Some("Font file does not exist".to_string()),
        cause: None,
        err_type: ApiErrorType::InvalidInput,
    })?;
    Ok(NamedFile::open(path)?)
}

#[cfg(test)]
mod testing {
    use std::{
        fs::read_dir,
        path::{Path, PathBuf},
    };

    use crate::routes::manga::TranslationResponse;

    pub fn is_valid_translation(s: &str) -> bool {
        let v: Result<TranslationResponse, _> = serde_json::from_str(s);
        v.is_ok()
    }

    fn search_directory_for_json_files(directory: &Path) -> Vec<PathBuf> {
        let mut json_files = Vec::new();

        if let Ok(entries) = read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        json_files.extend(search_directory_for_json_files(&path));
                    } else if let Some(extension) = path.extension() {
                        if extension == "json" {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if !is_valid_translation(&content) {
                                    json_files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        json_files
    }

    #[test]
    fn test_json_structure() {
        let directory = "data/mangas"; // Specify the directory to search
        let json_files = search_directory_for_json_files(&PathBuf::from(directory));
        assert_eq!(json_files.len(), 0)
    }
}
