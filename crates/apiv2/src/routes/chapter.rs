use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{ChapterInfoResponse, EditChapterRequest, IdRequest, NewChapterRequest},
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};
use chrono::DateTime;

use crate::{
    actions::chapter::ChapterActions,
    error::{ApiError, ApiResult},
};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/chapter")
        .service(
            apistos::web::resource("/add").route(
                apistos::web::put()
                    .to(add)
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
}
#[api_operation(
    tag = "chapter",
    summary = "Returns info about a chapter",
    description = r###""###
)]
pub(crate) async fn info(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<Json<ChapterInfoResponse>> {
    chapter_service.info(&data.id).await.map(Json)
}

#[api_operation(
    tag = "chapter",
    summary = "Adds a chapter to an existing chapter",
    description = r###""###
)]
pub(crate) async fn add(
    Json(chapter): Json<NewChapterRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<CreatedJson<u8>> {
    let release_date = chapter
        .release_date
        .map(|value| {
            i64::try_from(value)
                .ok()
                .and_then(DateTime::from_timestamp_millis)
                .ok_or(ApiError::invalid_input("Invalid release_date timestamp"))
        })
        .transpose()?;
    chapter_service
        .add(
            &chapter.manga_id,
            chapter.titles,
            chapter.episode,
            &chapter.version,
            chapter.images,
            chapter.tags,
            chapter.sources,
            release_date,
        )
        .await?;
    Ok(CreatedJson(0))
}

#[api_operation(
    tag = "chapter",
    summary = "Deletes a chapter",
    description = r###""###
)]
pub(crate) async fn delete(
    Json(data): Json<IdRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<Json<u8>> {
    chapter_service.delete(&data.id).await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "chapter",
    summary = "Modifies a chapter",
    description = r###""###
)]
pub(crate) async fn edit(
    Json(data): Json<EditChapterRequest>,
    chapter_service: Data<ChapterActions>,
) -> ApiResult<Json<u8>> {
    chapter_service.edit(data).await?;
    Ok(Json(200))
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use actix_web::web::{Data, Json};
    use api_structure::v1::NewChapterRequest;
    use db::{init_db, DbConfig, MemoryDbConfig};
    use storage::{MemStorage, StorageSystem};

    use crate::actions::chapter::ChapterActions;

    use super::*;

    async fn test_chapter_action() -> ChapterActions {
        let db = init_db(DbConfig::Memory(MemoryDbConfig {
            namespace: format!("ns_route_chapter_{}", helper::random_string(8)),
            database: format!("db_route_chapter_{}", helper::random_string(8)),
        }))
        .await
        .expect("memory db should initialize");

        let root =
            std::env::temp_dir().join(format!("apiv2-route-chapter-{}", helper::random_string(8)));
        tokio::fs::create_dir_all(&root)
            .await
            .expect("route chapter temp root should be created");
        let storage = Arc::new(
            StorageSystem::new(PathBuf::as_path(&root), Arc::new(MemStorage::new()))
                .await
                .expect("memory storage should initialize"),
        );

        ChapterActions {
            chapters: db.chapters,
            tags: db.tags,
            versions: db.versions,
            chapter_versions: db.chapter_versions,
            mangas: db.mangas,
            pages: db.pages,
            fs: storage,
        }
    }

    #[actix_web::test]
    async fn add_rejects_invalid_release_date_timestamp() {
        let service = Data::new(test_chapter_action().await);
        let response = add(
            Json(NewChapterRequest {
                manga_id: "manga".to_owned(),
                titles: vec!["chapter 1".to_owned()],
                episode: 1.0,
                version: "en".to_owned(),
                images: vec!["image-temp".to_owned()],
                tags: vec![],
                sources: vec!["https://source.example".to_owned()],
                release_date: Some(u64::MAX),
            }),
            service,
        )
        .await;

        assert!(matches!(response, Err(ApiError::InvalidInput(_))));
    }
}
