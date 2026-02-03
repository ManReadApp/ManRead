pub mod detail;
pub mod home;
mod reader;
pub mod search;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/manga")
        .service(detail::register())
        .service(home::register())
        .service(search::register())
        .service(reader::register())
}
