use apistos::web::{scope, Scope};

mod search;
pub fn register() -> Scope {
    scope("/tags").service(search::register())
}
