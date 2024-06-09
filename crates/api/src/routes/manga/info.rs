use crate::errors::{ApiError, ApiResult};
use crate::services::db::chapter::Chapter;
use crate::services::db::manga::MangaDBService;
use crate::services::db::manga_kind::MangaKindDBService;
use crate::services::db::tag::TagDBService;
use crate::services::db::user::UserDBService;
use crate::services::uri_service::UriService;
use actix_web::post;
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::protect;
use api_structure::auth::jwt::Claim;
use api_structure::auth::role::Role;
use api_structure::info::{ExternalSite, MangaInfoRequest, MangaInfoResponse, Visibility};
use api_structure::search::Status;
use rand::Rng;

#[post("/info")]
#[protect(
    any(
        "Role::Admin",
        "Role::CoAdmin",
        "Role::Moderator",
        "Role::Author",
        "Role::User"
    ),
    ty = "Role"
)]
pub async fn info(
    Json(req): Json<MangaInfoRequest>,
    manga: Data<MangaDBService>,
    tags_s: Data<TagDBService>,
    user: ReqData<Claim>,
    kind_s: Data<MangaKindDBService>,
    user_s: Data<UserDBService>,
    uri: Data<UriService>,
) -> ApiResult<Json<MangaInfoResponse>> {
    let manga = manga.get(req.manga_id.as_str()).await?;
    let kind = kind_s
        .get_kind(&manga.data.kind.thing.id().to_string())
        .await
        .ok_or(ApiError::db_error())?;
    let mut tags = vec![];

    for tag in manga.data.tags {
        tags.push(
            tags_s
                .get_tag(&tag.thing.id().to_string())
                .await
                .ok_or(ApiError::db_error())?
                .to_public(),
        );
    }

    let pos = rand::thread_rng().gen_range(0..manga.data.covers.len());

    let mut artists = vec![];
    for artist in manga.data.artists {
        artists.push(
            user_s
                .get_username(&artist.thing.id().to_string())
                .await
                .ok_or(ApiError::db_error())?,
        );
    }
    let mut authors = vec![];
    for author in manga.data.authors {
        authors.push(
            user_s
                .get_username(&author.thing.id().to_string())
                .await
                .ok_or(ApiError::db_error())?,
        );
    }
    let mut chapters = vec![];
    for chapter in manga.data.chapters {
        let v: Chapter = chapter
            .thing
            .get(&*tags_s.conn)
            .await?
            .ok_or(ApiError::db_error())?;
        let mut ctags = vec![];
        for tag in v.tags {
            ctags.push(
                tags_s
                    .get_tag(&tag.thing.id().to_string())
                    .await
                    .ok_or(ApiError::db_error())?
                    .to_public(),
            );
        }
        chapters.push(api_structure::info::Chapter {
            titles: v.titles,
            chapter: v.chapter,
            tags: ctags,
            sources: v.sources,
            release_date: v.release_date.map(|v| v.to_string()),
        });
    }

    Ok(Json(MangaInfoResponse {
        manga_id: manga.id.0.id.to_string(),
        titles: manga.data.titles,
        kind: kind.kind,
        description: manga.data.description,
        tags,
        status: Status::try_from(manga.data.status)?,
        visibility: Visibility::try_from(manga.data.visibility)?,
        my: manga.data.uploader.thing.id().to_string() == user.id,
        uploader: user_s
            .get_username(&manga.data.uploader.thing.id().to_string())
            .await
            .ok_or(ApiError::db_error())?,
        artists,
        authors,
        cover: pos as u32,
        cover_ext: manga.data.covers.get(pos).unwrap().clone(),
        chapters,
        sources: manga
            .data
            .sources
            .into_iter()
            .map(|url| {
                let icon_uri = uri.get_uri(url.as_str());
                ExternalSite { url, icon_uri }
            })
            .collect(),
        scraper: !manga.data.scraper.is_empty(),
        relations: manga
            .data
            .relations
            .into_iter()
            .map(|v| (v.thing.id().to_string(), "".to_string()))
            .collect(),
        favorite: false,
        progress: None,
    }))
}
