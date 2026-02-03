use actix_files::NamedFile;
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use apistos::api_operation;

use crate::error::{ApiError, ApiResult};

#[api_operation(
    tag = "image",
    summary = "Gets the cover of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    filename: Path<String>,
    http_req: HttpRequest,
    config: Data<crate::Config>,
) -> ApiResult<HttpResponse> {
    if filename.contains("/") {
        return Err(ApiError::InvalidImageId);
    }
    return Ok(
        NamedFile::open(config.root_folder.join("covers").join(filename.as_str()))?
            .into_response(&http_req),
    );
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/cover/{filename}").route(apistos::web::get().to(exec))
}
