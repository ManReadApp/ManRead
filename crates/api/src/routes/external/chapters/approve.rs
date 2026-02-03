use actix_web::web::Json;
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surrealdb::opt::PatchOp;
use surrealdb_extras::{RecordIdType, SurrealTableInfo};

use crate::{
    error::ApiResult,
    init::db::DB,
    models::scraper::{ScraperEntry, State},
};

#[api_operation(
    tag = "external",
    summary = "Decides which chapters will be scraped",
    description = r###""###
)]
pub(crate) async fn exec(Json(data): Json<Vec<ChangeRequest>>) -> ApiResult<CreatedJson<u8>> {
    for item in data {
        if let Some(v) = item.id.strip_prefix(&format!("{}:", ScraperEntry::name())) {
            let id: RecordIdType<ScraperEntry> = RecordIdType::from((ScraperEntry::name(), v));
            id.patch(&*DB, PatchOp::replace("/state", item.action))
                .await?;
        }
    }
    Ok(CreatedJson(0))
}

#[derive(Deserialize, Serialize, ApiComponent, JsonSchema)]
pub struct ChangeRequest {
    pub id: String,
    pub action: State,
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/approve").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
