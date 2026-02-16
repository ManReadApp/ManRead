use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use apistos::api_operation;
use storage::StorageSystem;

use crate::{
    error::{ApiError, ApiResult},
    routes::image::stream::stream,
};

#[api_operation(
    tag = "image",
    summary = "Gets the cover of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    filename: Path<String>,
    req: HttpRequest,
    storage: Data<StorageSystem>,
) -> ApiResult<HttpResponse> {
    if filename.contains("/") {
        return Err(ApiError::InvalidImageId);
    }
    let key = format!("covers/{filename}");
    let obj = storage.reader.get(&key, &Default::default()).await?;

    Ok(stream(&req, obj, true))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/cover/{filename}").route(apistos::web::get().to(exec))
}
