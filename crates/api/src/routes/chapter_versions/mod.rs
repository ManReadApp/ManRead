mod delete;
mod edit;
mod info;
mod list;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapter-versions")
        .service(list::register())
        .service(delete::register())
        .service(edit::register())
        .service(info::register())
}
