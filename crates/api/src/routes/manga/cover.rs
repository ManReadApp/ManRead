use crate::env::config::Config;
use crate::errors::ApiResult;
use actix_files::NamedFile;
use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web_grants::protect;
use api_structure::req::manga::cover::MangaCoverRequest;

#[post("/cover")]
#[protect(
    any(
        "api_structure::auth::role::Role::Admin",
        "api_structure::auth::role::Role::CoAdmin",
        "api_structure::auth::role::Role::Moderator",
        "api_structure::auth::role::Role::Author",
        "api_structure::auth::role::Role::User"
    ),
    ty = "api_structure::auth::role::Role"
)]
pub async fn cover_route(
    Json(data): Json<MangaCoverRequest>,
    config: Data<Config>,
) -> ApiResult<NamedFile> {
    Ok(NamedFile::open(
        config
            .root_folder
            .join("covers")
            .join(format!("{}.{}", data.manga_id, data.file_ext)),
    )?)
}
