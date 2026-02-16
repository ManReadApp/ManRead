use actix_web::{
    web::{Data, Json},
    HttpRequest, Responder,
};
use actix_web_grants::AuthorityGuard;
use api_structure::{v1::MangaReaderImageRequest, Permission};
use apistos::api_operation;
use storage::StorageSystem;

use crate::{
    error::{ApiError, ApiResult},
    routes::image::stream::stream,
};

#[api_operation(
    skip = true,
    tag = "image",
    summary = "Gets a specific image of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaReaderImageRequest>,
    req: HttpRequest,
    storage: Data<StorageSystem>,
) -> ApiResult<impl Responder> {
    if data.manga_id.contains("/") || data.chapter_id.contains("/") || data.version_id.contains("/")
    {
        return Err(ApiError::InvalidImageId);
    }
    let key = format!(
        "mangas/{}/{}/{}/{}.{}",
        data.manga_id, data.chapter_id, data.version_id, data.page, data.file_ext
    );
    let obj = storage.reader.get(&key, &Default::default()).await?;

    Ok(stream(&req, obj, false))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/page").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
