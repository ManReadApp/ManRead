use crate::{errors::ApiResult, services::db::tag::TagDBService};
use actix_web::{
    post,
    web::{Data, Json},
};
use actix_web_grants::protect;
use api_structure::{
    models::manga::tag::{Tag, TagSex},
    req::manga::tag::TagsRequest,
};

#[post("/tags")]
#[protect(
    any(
        "api_structure::models::auth::role::Role::Admin",
        "api_structure::models::auth::role::Role::CoAdmin",
        "api_structure::models::auth::role::Role::Moderator",
        "api_structure::models::auth::role::Role::Author",
        "api_structure::models::auth::role::Role::User"
    ),
    ty = "api_structure::models::auth::role::Role"
)]

pub async fn get_tags(
    Json(req): Json<TagsRequest>,
    tag_service: Data<TagDBService>,
) -> ApiResult<Json<Vec<Tag>>> {
    tag_service
        .search_tags(&req.query, req.limit, req.sex as u64)
        .await
        .map(|v| {
            Json(
                v.into_iter()
                    .map(|v| Tag {
                        tag: v.data.tag,
                        description: v.data.description,
                        sex: TagSex::from(v.data.sex),
                    })
                    .collect::<Vec<_>>(),
            )
        })
}
