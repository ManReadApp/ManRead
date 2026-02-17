use std::sync::Arc;

use actix_web::{middleware::Logger, App, HttpServer};
use apistos::{
    app::{BuildConfig, OpenApiWrapper as _},
    info::Info,
    spec::Spec,
    web, RapidocConfig, RedocConfig, ScalarConfig, SwaggerUIConfig,
};
use db::DbHandle;
use storage::StorageSystem;

use crate::{
    init::{app_data::init_app_data, env::Config, logger::log_url},
    routes,
};

#[cfg(feature = "cors")]
pub fn init_cors() -> actix_cors::Cors {
    #[cfg(all(feature = "cors", not(feature = "cors-permissive")))]
    return actix_cors::Cors::default()
        .allow_any_header()
        .allowed_methods(vec!["GET", "POST"])
        .supports_credentials()
        .max_age(3600);
    #[cfg(all(feature = "cors", feature = "cors-permissive"))]
    return actix_cors::Cors::permissive();
    #[cfg(not(any(feature = "cors", feature = "cors-permissive")))]
    unreachable!("this function should only be called when cors is activated")
}

pub fn init_server(
    port: u16,
    https_port: u16,
    config: Arc<Config>,
    fs: Arc<StorageSystem>,
    dbs: DbHandle,
) -> std::io::Result<actix_web::dev::Server> {
    log_url(&config);
    let app_data = move || init_app_data(config.clone(), fs.clone(), dbs.clone());
    #[cfg(feature = "https")]
    let ssl_builder = https::init_https(&config.root_folder)?;
    #[cfg(not(feature = "https"))]
    let _ = https_port;
    let hs = HttpServer::new(move || {
        let logger = Logger::default();
        let spec = Spec {
            info: Info {
                title: "An API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let app = App::new().wrap(logger);
        #[cfg(feature = "cors")]
        let app = app.wrap(init_cors());
        let app = app
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
            .service(web::redirect("/tip", "https://ko-fi.com/manread"));
        app.document(spec)
            .service(app_data().service(routes::register()))
            .build_with(
                "/openapi.json",
                BuildConfig::default()
                    .with(RapidocConfig::new(&"/rapidoc"))
                    .with(RedocConfig::new(&"/redoc"))
                    .with(ScalarConfig::new(&"/scalar"))
                    .with(SwaggerUIConfig::new(&"/swagger"))
                    .with(SwaggerUIConfig::new(&"/docs")),
            )
    });

    let hs = hs.bind(format!("0.0.0.0:{}", port))?;
    #[cfg(feature = "https")]
    let hs = hs.bind_openssl(format!("0.0.0.0:{}", https_port), ssl_builder)?;
    Ok(hs.run())
}
