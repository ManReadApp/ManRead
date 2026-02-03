mod delete;
mod list;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/errors")
        .service(list::register())
        .service(delete::register())
}
