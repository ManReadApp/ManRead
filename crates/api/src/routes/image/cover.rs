use actix_files::NamedFile;
use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::manga::cover::MangaCoverRequest};
use apistos::api_operation;

use crate::{
    error::{ApiError, ApiResult},
    Config,
};

#[api_operation(
    skip = true,
    tag = "image",
    summary = "Gets the cover of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaCoverRequest>,
    config: Data<Config>,
) -> ApiResult<NamedFile> {
    if data.manga_id.contains("/") {
        return Err(ApiError::InvalidImageId);
    }
    println!(
        "{}",
        config
            .root_folder
            .join("covers")
            .join(format!("{}.{}", data.manga_id, data.file_ext))
            .display()
    );
    Ok(NamedFile::open(
        config
            .root_folder
            .join("covers")
            .join(format!("{}.{}", data.manga_id, data.file_ext)),
    )?)
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/cover").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
