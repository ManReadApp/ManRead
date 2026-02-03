use actix_files::NamedFile;
use actix_web::{
    http::header,
    web::{Data, Json},
    Responder,
};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::reader::image::MangaReaderImageRequest};
use apistos::api_operation;

use crate::{
    error::{ApiError, ApiResult},
    Config,
};

#[api_operation(
    skip = true,
    tag = "image",
    summary = "Gets a specific image of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaReaderImageRequest>,
    config: Data<Config>,
) -> ApiResult<impl Responder> {
    if data.manga_id.contains("/") || data.chapter_id.contains("/") || data.version_id.contains("/")
    {
        return Err(ApiError::InvalidImageId);
    }
    Ok(NamedFile::open(
        config
            .root_folder
            .join("mangas")
            .join(data.manga_id)
            .join(data.chapter_id)
            .join(data.version_id)
            .join(format!("{}.{}", data.page, data.file_ext)),
    )?
    .use_last_modified(false)
    .customize()
    .insert_header((
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate, max-age=0",
    ))
    .insert_header((header::PRAGMA, "no-cache"))
    .insert_header((header::EXPIRES, "0")))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/page").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
