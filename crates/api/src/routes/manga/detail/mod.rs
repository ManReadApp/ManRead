pub mod create;
pub mod delete;
pub mod edit;
mod info;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/detail")
        .service(create::register())
        .service(delete::register())
        .service(edit::register())
        .service(info::register())
}
