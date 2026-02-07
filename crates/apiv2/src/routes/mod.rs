use actix_web::{web, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use apistos::web::{scope, Scope};

use crate::actions::crytpo::validator;

mod admin;
mod auth;
mod img;

pub fn register() -> Scope {
    apistos::web::scope("/v1")
        .service(auth::register())
        // .service(scope("/image-no-auth").service(image::cover_img::register()))
        .service(
            scope("")
                .wrap(HttpAuthentication::bearer(validator))
                // .service(chapter::register())
                // .service(chapter_versions::register())
                // .service(image::register())
                // .service(manga::register())
                // .service(user::register())
                // .service(admin::register())
                // .service(external::register())
                // .service(lists::register())
                // .service(kind::register())
                // .service(errors::register())
                // .service(tags::register())
                .default_service(web::route().to(not_found)),
        )
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("404 Not Found")
}
