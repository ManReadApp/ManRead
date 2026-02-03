use actix_web_httpauth::middleware::HttpAuthentication;
use apistos::web::scope;

use super::validator;

mod create;
mod login;
mod logout;
mod refresh;
mod reset_password;
mod verify;
mod verify_reset_password;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/auth")
        .service(login::register())
        .service(logout::register())
        .service(create::register())
        .service(refresh::register())
        .service(reset_password::register())
        .service(verify_reset_password::register())
        .service(
            scope("")
                .wrap(HttpAuthentication::bearer(validator))
                .service(verify::register()),
        )
}
