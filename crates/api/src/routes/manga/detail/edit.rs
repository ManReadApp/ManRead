use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::manga::edit::EditMangaRequest};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::ApiResult,
    models::{
        kind::KindDBService, manga::MangaDBService, tag::TagDBService, user::UserDBService,
        version::VersionDBService,
    },
};

use super::create::prepare;

#[api_operation(tag = "manga", summary = "Edit a manga", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<EditMangaRequest>,
    manga_service: Data<MangaDBService>,
    kind_service: Data<KindDBService>,
    tag_service: Data<TagDBService>,
    version_service: Data<VersionDBService>,
    user_service: Data<UserDBService>,
) -> ApiResult<CreatedJson<u16>> {
    let (tags, scrapers, artists, authors, publishers) = prepare(
        data.tags,
        data.scrapers,
        data.artists,
        data.authors,
        data.publishers,
        tag_service,
        version_service,
        user_service,
    )
    .await?;
    let kind = kind_service.get_or_create(&data.kind).await?;
    manga_service
        .update(
            &data.manga_id,
            data.names,
            data.status,
            data.description,
            tags,
            authors,
            artists,
            publishers,
            data.sources,
            scrapers,
            kind,
        )
        .await?;
    manga_service.regenerate_tags(&data.manga_id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/edit").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Create)),
    )
}
