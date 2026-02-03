use apistos::web::{scope, Scope};

mod list;
pub fn register() -> Scope {
    scope("/kind").service(list::register())
}
