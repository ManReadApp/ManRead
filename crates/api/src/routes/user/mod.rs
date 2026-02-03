pub mod delete;
pub mod edit;
pub mod info;
pub mod list;
pub mod search;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/user")
        .service(delete::register())
        .service(edit::register())
        .service(info::register())
        .service(list::register())
        .service(search::register())
}
