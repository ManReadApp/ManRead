mod create;
mod delete;
mod list;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/verify")
        .service(create::register())
        .service(delete::register())
        .service(list::register())
}
