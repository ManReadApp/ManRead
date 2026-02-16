use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    search::{HomeResponse, SearchRequest, SearchResponse_},
    v1::{AddMangaRequest, Claim, EditMangaRequest, IdRequest, MangaInfoResponse},
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{actions::manga::MangaActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/manga")
        .service(
            apistos::web::scope("/detail")
                .service(
                    apistos::web::resource("/create").route(
                        apistos::web::put()
                            .to(create)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/delete").route(
                        apistos::web::delete()
                            .to(delete)
                            .guard(AuthorityGuard::new(Permission::RequestDelete)),
                    ),
                )
                .service(
                    apistos::web::resource("/edit").route(
                        apistos::web::put()
                            .to(edit)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/info").route(
                        apistos::web::post()
                            .to(info)
                            .guard(AuthorityGuard::new(Permission::Read)),
                    ),
                ),
        )
        .service(
            apistos::web::resource("/home").route(
                apistos::web::post()
                    .to(home)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/search").route(
                apistos::web::post()
                    .to(search)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
}

#[api_operation(
    tag = "manga",
    summary = "Creates a manga",
    description = r###"Returns the manga id"###
)]
pub(crate) async fn create(
    Json(data): Json<AddMangaRequest>,
    manga_service: Data<MangaActions>,
    uploader: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service.create(data, &uploader.id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "manga",
    summary = "Deletes a manga",
    description = r###"Doenst really delete the manga. just sets the visibility to admin review for delete/make inacessible"###
)]
pub(crate) async fn delete(
    Json(data): Json<IdRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service.delete(&data.id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(tag = "manga", summary = "Edit a manga", description = r###""###)]
pub(crate) async fn edit(
    Json(data): Json<EditMangaRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<CreatedJson<u16>> {
    manga_service.edit(data).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "manga",
    summary = "All details about a manga",
    description = r###""###
)]
pub(crate) async fn info(
    Json(data): Json<IdRequest>,
    manga_service: Data<MangaActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<MangaInfoResponse>> {
    manga_service.info(data.id, &user.id).await.map(Json)
}

#[api_operation(
    tag = "manga",
    summary = "Gets all the info for the manga home",
    description = r###""###
)]
pub(crate) async fn home(
    manga_service: Data<MangaActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<HomeResponse>> {
    manga_service.home(&user.id).await.map(Json)
}

#[api_operation(tag = "manga", summary = "Search for manga", description = r###""###)]
pub(crate) async fn search(
    Json(data): Json<SearchRequest>,
    search_service: Data<MangaActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<SearchResponse_>> {
    search_service
        .search(data, &user.id)
        .await
        .map(|v| SearchResponse_ {
            items: v.0,
            max: v.1,
        })
        .map(Json)
}
