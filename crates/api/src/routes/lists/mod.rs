use apistos::web::{scope, Scope};

pub mod add;
pub mod delete;
pub mod list;
pub mod namespace;

pub fn register() -> Scope {
    scope("/lists")
        .service(add::register())
        .service(delete::register())
        .service(list::register())
        .service(namespace::register())
}
