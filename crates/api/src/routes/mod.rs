use std::sync::Arc;

use actix_web::{
    dev::ServiceRequest,
    web::{self, Data},
    HttpMessage as _, HttpResponse,
};
use actix_web_grants::authorities::AttachAuthorities as _;
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use api_structure::models::auth::jwt::JwtType;
use apistos::web::scope;

use crate::{
    services::{auth::CryptoService, file::FileService},
    Config,
};

pub mod admin;
mod auth;
pub mod chapter;
pub mod chapter_versions;
mod errors;
mod external;
pub mod image;
mod kind;
pub mod lists;
pub mod manga;
mod tags;
pub mod user;

pub async fn validator(
    req: ServiceRequest,
    cred: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let secret = req
        .app_data::<Data<CryptoService>>()
        .expect("CryptoService is missing");
    match secret.get_claim(cred.token()) {
        Ok(v) => {
            {
                if matches!(v.jwt_type, JwtType::AccessToken) {
                    req.attach(v.role.get_permissions());
                }
                let mut ext = req.extensions_mut();
                ext.insert(v);
            }
            Ok(req)
        }
        Err(e) => Err((e.into(), req)),
    }
}

pub fn register(config: Arc<Config>) -> apistos::web::Scope {
    apistos::web::scope("/v1")
        .app_data(Data::new(FileService::new(config)))
        .service(auth::register())
        .service(scope("/image-no-auth").service(image::cover_img::register()))
        .service(
            scope("")
                .wrap(HttpAuthentication::bearer(validator))
                .service(chapter::register())
                .service(chapter_versions::register())
                .service(image::register())
                .service(manga::register())
                .service(user::register())
                .service(admin::register())
                .service(external::register())
                .service(lists::register())
                .service(kind::register())
                .service(errors::register())
                .service(tags::register())
                .default_service(web::route().to(not_found)),
        )
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("404 Not Found")
}
