use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use actix_web_grants::AuthorityGuard;
use api_structure::{v1::MangaCoverRequest, Permission};
use apistos::api_operation;
use storage::StorageSystem;

use crate::{
    error::{ApiError, ApiResult},
    routes::image::stream::stream,
};

#[api_operation(
    skip = true,
    tag = "image",
    summary = "Gets the cover of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaCoverRequest>,
    req: HttpRequest,
    storage: Data<StorageSystem>,
) -> ApiResult<HttpResponse> {
    if data.manga_id.contains("/") {
        return Err(ApiError::InvalidImageId);
    }
    let key = format!("covers/{}.{}", data.manga_id, data.file_ext);
    let obj = storage.reader.get(&key, &Default::default()).await?;

    Ok(stream(&req, obj, true))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/cover").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
