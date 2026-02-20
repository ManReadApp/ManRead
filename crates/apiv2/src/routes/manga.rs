use actix_web::{
    http::header::{
        ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE,
    },
    web::{Data, Json, ReqData},
    HttpRequest, HttpResponse,
};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    search::{HomeResponse, SearchRequest, SearchResponse_},
    v1::{
        AddMangaArtRequest, AddMangaCoverRequest, AddMangaRelationRequest, AddMangaRequest, Claim,
        ConfirmMangaDeleteRequest, EditMangaRequest, IdRequest, MangaInfoResponse,
        RemoveMangaArtRequest, RemoveMangaCoverRequest, RemoveMangaRelationRequest,
        SetMangaVolumeRangeRequest,
    },
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::{
    actions::manga::{MangaActions, VolumeRange},
    error::{ApiError, ApiResult},
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
                    apistos::web::resource("/export").route(
                        apistos::web::post()
                            .to(export)
                            .guard(AuthorityGuard::new(Permission::Read)),
                    ),
                )
                .service(
                    apistos::web::resource("/restore").route(
                        apistos::web::post()
                            .to(restore)
                            .guard(AuthorityGuard::new(Permission::Create)),
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
    let _ = manga_service.create(data, &uploader.id).await?;
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

#[api_operation(skip = true)]
pub(crate) async fn export(
    Json(data): Json<IdRequest>,
    manga_service: Data<MangaActions>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let manga_id = data.id;
    let prepared = manga_service.prepare_export(&manga_id).await?;
    let total_len = prepared.total_len();

    let range = match req.headers().get(RANGE) {
        None => None,
        Some(raw) => {
            let Some(range) = parse_single_range(raw.to_str().ok(), total_len) else {
                return Ok(HttpResponse::RangeNotSatisfiable()
                    .insert_header((ACCEPT_RANGES, "bytes"))
                    .insert_header((CONTENT_RANGE, format!("bytes */{total_len}")))
                    .finish());
            };
            Some(range)
        }
    };

    let output = prepared.into_stream(&manga_service.fs, range).await?;
    let (start, end) = output.range;
    let content_len = end - start + 1;

    let mut resp = if range.is_some() {
        let mut v = HttpResponse::PartialContent();
        v.insert_header((CONTENT_RANGE, format!("bytes {start}-{end}/{total_len}")));
        v
    } else {
        HttpResponse::Ok()
    };

    Ok(resp
        .insert_header((CONTENT_TYPE, "application/octet-stream"))
        .insert_header((ACCEPT_RANGES, "bytes"))
        .insert_header((CONTENT_LENGTH, content_len.to_string()))
        .insert_header((
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.mrmang\"", manga_id),
        ))
        .streaming(output.stream))
}

#[derive(Default, Debug, Deserialize)]
struct RestoreUploadGroup {
    metadata_id: Option<String>,
    images: BTreeMap<usize, String>,
}

fn parse_restore_upload_response(payload: Vec<Vec<String>>) -> ApiResult<(String, Vec<String>)> {
    if payload.is_empty() {
        return Err(ApiError::invalid_input("upload response cannot be empty"));
    }

    let mut groups: HashMap<String, RestoreUploadGroup> = HashMap::new();
    for row in payload {
        if row.len() != 2 {
            return Err(ApiError::invalid_input(
                "each upload response entry must have [name, id]",
            ));
        }
        let Some(name) = row.first().map(|v| v.trim()) else {
            return Err(ApiError::invalid_input("missing upload response name"));
        };
        let Some(id) = row.get(1).map(|v| v.trim()) else {
            return Err(ApiError::invalid_input("missing upload response id"));
        };
        if name.is_empty() || id.is_empty() {
            return Err(ApiError::invalid_input(
                "upload response name and id cannot be empty",
            ));
        }

        let Some((base_name, suffix)) = name.rsplit_once('#') else {
            continue;
        };
        if base_name.is_empty() {
            continue;
        }

        let group = groups.entry(base_name.to_owned()).or_default();
        match suffix {
            "meta" => {
                if group.metadata_id.replace(id.to_owned()).is_some() {
                    return Err(ApiError::invalid_input(
                        "upload response contains duplicate metadata entries",
                    ));
                }
            }
            image_suffix => {
                let Some(image_index) = image_suffix.strip_prefix('i') else {
                    continue;
                };
                let index = image_index
                    .parse::<usize>()
                    .map_err(|_| ApiError::invalid_input("invalid image index in upload name"))?;
                if group.images.insert(index, id.to_owned()).is_some() {
                    return Err(ApiError::invalid_input(
                        "upload response contains duplicate image entries",
                    ));
                }
            }
        }
    }

    let mut groups_with_meta = groups
        .into_iter()
        .filter(|(_, group)| group.metadata_id.is_some())
        .collect::<Vec<_>>();

    if groups_with_meta.len() != 1 {
        return Err(ApiError::invalid_input(
            "upload response must contain exactly one export metadata entry",
        ));
    }
    let (_, group) = groups_with_meta
        .pop()
        .ok_or_else(|| ApiError::invalid_input("missing export metadata entry"))?;
    let metadata_id = group
        .metadata_id
        .ok_or_else(|| ApiError::invalid_input("missing export metadata entry"))?;
    if group.images.is_empty() {
        return Ok((metadata_id, vec![]));
    }

    let max_index = group.images.keys().copied().max().unwrap_or_default();
    let mut image_ids = vec![None; max_index + 1];
    for (index, image_id) in group.images {
        image_ids[index] = Some(image_id);
    }
    if image_ids.iter().any(|entry| entry.is_none()) {
        return Err(ApiError::invalid_input(
            "upload response has missing image index entries",
        ));
    }

    Ok((
        metadata_id,
        image_ids
            .into_iter()
            .map(|entry| entry.unwrap_or_default())
            .collect(),
    ))
}

#[api_operation(
    tag = "manga",
    summary = "Restores an uploaded manga export bundle",
    description = r###"Accepts the raw `/image/upload` response for a `.mrmang` upload."###
)]
pub(crate) async fn restore(
    Json(payload): Json<Vec<Vec<String>>>,
    manga_service: Data<MangaActions>,
    user: ReqData<Claim>,
) -> ApiResult<Json<u8>> {
    let (metadata_id, image_ids) = parse_restore_upload_response(payload)?;
    manga_service
        .restore(&metadata_id, image_ids, &user.id)
        .await?;
    Ok(Json(200))
}

fn parse_single_range(value: Option<&str>, total: u64) -> Option<(u64, u64)> {
    let value = value?;
    let spec = value.strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    if start.is_empty() {
        let suffix: u64 = end.parse().ok()?;
        if suffix == 0 {
            return None;
        }
        let start = total.saturating_sub(suffix);
        return Some((start, total.saturating_sub(1)));
    }

    let start: u64 = start.parse().ok()?;
    if start >= total {
        return None;
    }
    let end = if end.is_empty() {
        total - 1
    } else {
        end.parse::<u64>().ok()?.min(total - 1)
    };
    if start > end {
        return None;
    }
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::{parse_restore_upload_response, parse_single_range};

    #[test]
    fn parses_valid_ranges() {
        assert_eq!(parse_single_range(Some("bytes=0-9"), 100), Some((0, 9)));
        assert_eq!(parse_single_range(Some("bytes=10-"), 100), Some((10, 99)));
        assert_eq!(parse_single_range(Some("bytes=-10"), 100), Some((90, 99)));
        assert_eq!(
            parse_single_range(Some("bytes=95-200"), 100),
            Some((95, 99))
        );
    }

    #[test]
    fn rejects_invalid_ranges() {
        assert_eq!(parse_single_range(Some("bytes=100-100"), 100), None);
        assert_eq!(parse_single_range(Some("bytes=9-1"), 100), None);
        assert_eq!(parse_single_range(Some("bytes=0-1,4-5"), 100), None);
        assert_eq!(parse_single_range(Some("units=0-1"), 100), None);
        assert_eq!(parse_single_range(Some("bytes=-0"), 100), None);
    }

    #[test]
    fn parse_restore_upload_response_extracts_metadata_and_images() {
        let payload = vec![
            vec!["bundle.mrmang#meta".to_owned(), "meta-id".to_owned()],
            vec!["bundle.mrmang#i1".to_owned(), "image-1".to_owned()],
            vec!["bundle.mrmang#i0".to_owned(), "image-0".to_owned()],
        ];

        let parsed =
            parse_restore_upload_response(payload).expect("payload should parse successfully");
        assert_eq!(parsed.0, "meta-id");
        assert_eq!(parsed.1, vec!["image-0".to_owned(), "image-1".to_owned()]);
    }

    #[test]
    fn parse_restore_upload_response_rejects_sparse_images() {
        let payload = vec![
            vec!["bundle.mrmang#meta".to_owned(), "meta-id".to_owned()],
            vec!["bundle.mrmang#i1".to_owned(), "image-1".to_owned()],
        ];

        assert!(parse_restore_upload_response(payload).is_err());
    }

    #[test]
    fn parse_restore_upload_response_rejects_multiple_meta_groups() {
        let payload = vec![
            vec!["bundle-a.mrmang#meta".to_owned(), "meta-a".to_owned()],
            vec!["bundle-b.mrmang#meta".to_owned(), "meta-b".to_owned()],
        ];

        assert!(parse_restore_upload_response(payload).is_err());
    }
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
    manga_service
        .add_cover(&data.manga_id, &data.file_id)
        .await?;
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
    manga_service
        .set_volume_range(&data.manga_id, ranges)
        .await?;
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
