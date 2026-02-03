use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::role::Permission,
    req::IdRequest,
    resp::version::info::{ChapterVersion, Page},
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{page::PageDBService, version_link::ChapterVersionDBService},
};

#[api_operation(
    tag = "chapter-versions",
    summary = "Gets reader information from a chapter version",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    version_service: Data<ChapterVersionDBService>,
    page_service: Data<PageDBService>,
) -> ApiResult<CreatedJson<ChapterVersion>> {
    let info = version_service.get(&data.id).await?;
    let pages = page_service.get(info.pages).await?;
    Ok(CreatedJson(ChapterVersion {
        pages: pages
            .into_iter()
            .map(|v| {
                (
                    v.data.page,
                    Page {
                        page: v.data.page,
                        id: v.id.id().to_string(),
                        width: v.data.width,
                        height: v.data.height,
                        ext: v.data.ext,
                    },
                )
            })
            .collect(),
        link: info.link,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/info").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
