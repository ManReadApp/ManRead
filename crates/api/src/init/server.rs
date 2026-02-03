use std::sync::Arc;

use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self},
    App, HttpServer,
};
use apistos::{
    app::{BuildConfig, OpenApiWrapper as _},
    info::Info,
    spec::Spec,
    web::Scope,
    RapidocConfig, RedocConfig, ScalarConfig, SwaggerUIConfig,
};

use crate::routes;

use super::{cors::init_cors, Config, HttpsBuilderType};

pub fn init_server(
    port: u16,
    https_port: u16,
    ssl_builder: HttpsBuilderType,
    config: Arc<Config>,
    app_data_scope: impl Fn() -> Scope + Send + 'static + Clone,
) -> std::io::Result<Server> {
    #[cfg(not(feature = "https"))]
    {
        let _ = ssl_builder;
        let _ = https_port;
    }
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
            .service(app_data_scope().service(routes::register(config.clone())))
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
