use std::cmp::Ordering;

use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{jwt::Claim, role::Permission},
        manga::{
            chapter::{Chapter, ExternalSite},
            status::Status,
            visiblity::Visibility,
        },
    },
    req::{manga::add::Scrapers, IdRequest},
    resp::manga::info::MangaInfoResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::{
        chapter::ChapterDBService, kind::KindDBService, lists::ListDBService,
        manga::MangaDBService, tag::TagDBService, user::UserDBService,
    },
};

#[api_operation(
    tag = "manga",
    summary = "All details about a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    manga_service: Data<MangaDBService>,
    list_service: Data<ListDBService>,
    user_service: Data<UserDBService>,
    kind_service: Data<KindDBService>,
    tag_service: Data<TagDBService>,
    chapter_service: Data<ChapterDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<MangaInfoResponse>> {
    let manga = manga_service.get(&data.id).await?;
    let chapters_ = chapter_service
        .get_simple(manga.chapters.into_iter())
        .await?;
    let mut chapters = vec![];
    for v in chapters_ {
        chapters.push(Chapter {
            id: v.id.id().to_string(),
            titles: v.data.titles,
            chapter: v.data.chapter,
            tags: tag_service
                .get_tags(v.data.tags.into_iter().map(|v| v.thing.id().to_string()))
                .await?,
            sources: v.data.sources,
            release_date: v.data.release_date.map(|v| v.to_string()),
        });
    }
    chapters.sort_by(|a, b| a.chapter.partial_cmp(&b.chapter).unwrap_or(Ordering::Equal));
    let resp = MangaInfoResponse {
        titles: manga.titles,
        kind: kind_service.get_name(manga.kind.clone()).await?,
        description: manga.description,
        tags: tag_service
            .get_tags(manga.tags.into_iter().map(|v| v.thing.id().to_string()))
            .await?,
        status: Status::try_from(manga.status).unwrap(),
        visibility: Visibility::try_from(manga.visibility).unwrap(),
        uploader: user_service
            .get_name_by_id(manga.uploader.clone())
            .await?
            .data,
        my: manga
            .artists
            .iter()
            .find(|v| v.thing.id().to_string() == user.id)
            .is_some()
            || manga
                .authors
                .iter()
                .find(|v| v.thing.id().to_string() == user.id)
                .is_some()
            || manga
                .publishers
                .iter()
                .find(|v| v.thing.id().to_string() == user.id)
                .is_some()
            || manga.uploader.thing.id().to_string() == user.id,
        artists: user_service
            .get_name_from_ids(manga.artists.into_iter())
            .await?,
        authors: user_service
            .get_name_from_ids(manga.authors.into_iter())
            .await?,
        publishers: user_service
            .get_name_from_ids(manga.publishers.into_iter())
            .await?,
        cover_ext: manga.covers,
        sources: manga
            .sources
            .into_iter()
            .map(|v| ExternalSite {
                url: v,
                //TODO: get icon uri
                icon_uri: "".to_owned(),
            })
            .collect(),
        relations: manga_service
            .get_names(manga.relations.into_iter(), vec!["en".to_owned()])
            .await?,
        scraper: manga.scraper.iter().any(|v| v.enabled),
        scrapers: {
            let mut scrapers = Vec::new();

            for v in manga.scraper {
                let target = v.target.get(&*DB).await?.ok_or(ApiError::NotFoundInDB)?;

                scrapers.push(Scrapers {
                    channel: target.data.name,
                    url: v.url,
                });
            }
            scrapers
        },
        favorite: list_service.is_favorite(&data.id, &user.id).await,
        progress: list_service.is_reading(&data.id, &user.id).await,
        chapters,
        manga_id: data.id,
    };
    Ok(CreatedJson(resp))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/info").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
