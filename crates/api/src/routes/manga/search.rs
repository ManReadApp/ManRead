use crate::errors::ApiResult;
use crate::routes::manga::home::format;
use crate::services::db::manga::MangaDBService;
use crate::services::db::manga_kind::MangaKindDBService;
use crate::services::db::tag::TagDBService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::protect;
use api_structure::auth::jwt::Claim;
use api_structure::auth::role::Role;
use api_structure::req::manga::search::SearchRequest;
use api_structure::resp::manga::search::SearchResponse;

#[post("/search")]
#[protect(
    any(
        "Role::Admin",
        "Role::CoAdmin",
        "Role::Moderator",
        "Role::Author",
        "Role::User"
    ),
    ty = "Role"
)]
async fn search(
    Json(request): Json<SearchRequest>,
    manga: Data<MangaDBService>,
    tags: Data<TagDBService>,
    user: ReqData<Claim>,
    user_service: Data<UserDBService>,
    kind_service: Data<MangaKindDBService>,
    tag_service: Data<TagDBService>,
) -> ApiResult<Json<Vec<SearchResponse>>> {
    Ok(Json(
        format(
            manga
                .search(
                    request,
                    &user.id,
                    &user_service,
                    &kind_service,
                    &tag_service,
                )
                .await?,
            &tags,
        )
        .await?,
    ))
}
