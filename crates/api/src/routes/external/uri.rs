use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use manga_scraper::init::Services;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, ApiComponent, JsonSchema)]
struct GetUriRequest {
    url: String,
}
#[api_operation(
    tag = "external",
    summary = "Gets the uri of a website",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<GetUriRequest>,
    services: Data<Services>,
) -> CreatedJson<Option<String>> {
    let v = services.get(&data.url);
    CreatedJson(v.map(|v| v.uri.to_owned()))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/uri").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
