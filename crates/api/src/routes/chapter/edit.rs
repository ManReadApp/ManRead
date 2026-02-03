use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation};

#[api_operation(
    tag = "chapter",
    summary = "Modifies a chapter",
    description = r###""###
)]
pub(crate) async fn exec() -> CreatedJson<String> {
    //TODO: impl
    CreatedJson("Hello World".to_owned())
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/edit").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
