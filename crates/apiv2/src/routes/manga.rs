use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    search::{HomeResponse, SearchRequest, SearchResponse_},
    v1::{
        AddMangaArtRequest, AddMangaCoverRequest, AddMangaRelationRequest, AddMangaRequest,
        Claim, ConfirmMangaDeleteRequest, EditMangaRequest, IdRequest, MangaInfoResponse,
        RemoveMangaArtRequest, RemoveMangaCoverRequest, RemoveMangaRelationRequest,
        SetMangaVolumeRangeRequest,
    },
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    actions::manga::{MangaActions, VolumeRange},
    error::ApiResult,
};

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
                )
                .service(
                    apistos::web::resource("/add-cover").route(
                        apistos::web::put()
                            .to(add_cover)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/remove-cover").route(
                        apistos::web::delete()
                            .to(remove_cover)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/add-art").route(
                        apistos::web::put()
                            .to(add_art)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/remove-art").route(
                        apistos::web::delete()
                            .to(remove_art)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/confirm-delete").route(
                        apistos::web::delete()
                            .to(confirm_delete)
                            .guard(AuthorityGuard::new(Permission::Review)),
                    ),
                )
                .service(
                    apistos::web::resource("/set-volume-range").route(
                        apistos::web::put()
                            .to(set_volume_range)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/add-relation").route(
                        apistos::web::put()
                            .to(add_relation)
                            .guard(AuthorityGuard::new(Permission::Create)),
                    ),
                )
                .service(
                    apistos::web::resource("/remove-relation").route(
                        apistos::web::delete()
                            .to(remove_relation)
                            .guard(AuthorityGuard::new(Permission::Create)),
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
) -> ApiResult<Json<u8>> {
    manga_service.delete(&data.id).await?;
    Ok(Json(200))
}

#[api_operation(tag = "manga", summary = "Edit a manga", description = r###""###)]
pub(crate) async fn edit(
    Json(data): Json<EditMangaRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service.edit(data).await?;
    Ok(Json(200))
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
    summary = "Adds a cover image to manga",
    description = r###""###
)]
pub(crate) async fn add_cover(
    Json(data): Json<AddMangaCoverRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service.add_cover(&data.manga_id, &data.file_id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "manga",
    summary = "Removes a cover image from manga",
    description = r###""###
)]
pub(crate) async fn remove_cover(
    Json(data): Json<RemoveMangaCoverRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service
        .remove_cover(&data.manga_id, data.cover_index as usize)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "manga",
    summary = "Adds an art image to manga",
    description = r###""###
)]
pub(crate) async fn add_art(
    Json(data): Json<AddMangaArtRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<CreatedJson<u8>> {
    manga_service.add_art(&data.manga_id, &data.file_id).await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "manga",
    summary = "Removes an art image from manga",
    description = r###""###
)]
pub(crate) async fn remove_art(
    Json(data): Json<RemoveMangaArtRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service
        .remove_art(&data.manga_id, data.art_index as usize)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "manga",
    summary = "Confirms a manga delete request",
    description = r###""###
)]
pub(crate) async fn confirm_delete(
    Json(data): Json<ConfirmMangaDeleteRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service.confirm_delete(&data.manga_id).await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "manga",
    summary = "Sets display volume ranges for manga chapters",
    description = r###""###
)]
pub(crate) async fn set_volume_range(
    Json(data): Json<SetMangaVolumeRangeRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    let ranges = data
        .ranges
        .into_iter()
        .map(|v| VolumeRange {
            start: v.start,
            end: v.end,
            title: v.title,
        })
        .collect();
    manga_service.set_volume_range(&data.manga_id, ranges).await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "manga",
    summary = "Adds a relation between two mangas",
    description = r###""###
)]
pub(crate) async fn add_relation(
    Json(data): Json<AddMangaRelationRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service
        .add_relation(&data.manga_id, &data.relation_id)
        .await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "manga",
    summary = "Removes a relation between two mangas",
    description = r###""###
)]
pub(crate) async fn remove_relation(
    Json(data): Json<RemoveMangaRelationRequest>,
    manga_service: Data<MangaActions>,
) -> ApiResult<Json<u8>> {
    manga_service
        .remove_relation(&data.manga_id, &data.relation_id)
        .await?;
    Ok(Json(200))
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
