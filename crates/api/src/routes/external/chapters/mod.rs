mod approve;
mod list;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapters")
        .service(approve::register())
        .service(list::register())
}
