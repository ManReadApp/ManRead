pub mod verify;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/admin").service(verify::register())
}
