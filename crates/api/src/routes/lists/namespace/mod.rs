use apistos::web::{scope, Scope};

pub mod add;
pub mod delete;

pub fn register() -> Scope {
    scope("/{list}")
        .service(add::register())
        .service(delete::register())
}
