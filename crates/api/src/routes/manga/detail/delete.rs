use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{auth::role::Permission, manga::visiblity::Visibility},
    req::IdRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::manga::MangaDBService};

#[api_operation(
    tag = "manga",
    summary = "Deletes a manga",
    description = r###"Doenst really delete the manga. just sets the visibility to admin review for delete/make inacessible"###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    manga_service: Data<MangaDBService>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service
        .set_visibility(data.id, Visibility::AdminReview)
        .await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/delete").route(
        apistos::web::delete()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::RequestDelete)),
    )
}
