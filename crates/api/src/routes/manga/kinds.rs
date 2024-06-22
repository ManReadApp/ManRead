use actix_web::{
    post,
    web::{Data, Json},
};
use actix_web_grants::protect;
use api_structure::resp::manga::KindsResponse;

use crate::{errors::ApiResult, services::db::manga_kind::MangaKindDBService};

#[post("/kinds")]
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
pub async fn get_kinds(kind_service: Data<MangaKindDBService>) -> ApiResult<Json<KindsResponse>> {
    kind_service.all().await.map(Json)
}
