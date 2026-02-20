use actix_web::{web, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use apistos::web::{scope, Scope};

use crate::actions::crytpo::validator;

mod auth;
mod chapter;
mod chapter_versions;
mod character;
mod image;
mod kind;
mod lists;
mod manga;
mod reader;
mod tags;
mod token;
mod user;

pub fn register() -> Scope {
    apistos::web::scope("/v1")
        .service(auth::register())
        .service(scope("/image-no-auth").service(image::cover_img::register()))
        .service(
            scope("")
                .wrap(HttpAuthentication::bearer(validator))
                // .service(external::register())
                .service(chapter::register())
                .service(character::register())
                .service(chapter_versions::register())
                .service(image::register())
                .service(token::register())
                .service(kind::register())
                .service(reader::register())
                .service(manga::register())
                .service(user::register())
                .service(lists::register())
                .service(tags::register())
                .default_service(web::route().to(not_found)),
        )
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("404 Not Found")
}
