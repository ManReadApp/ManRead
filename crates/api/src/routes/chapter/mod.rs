pub mod add;
pub mod delete;
pub mod edit;
mod info;
mod save_progress;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapter")
        .service(add::register())
        .service(delete::register())
        .service(edit::register())
        .service(save_progress::register())
        .service(info::register())
}
