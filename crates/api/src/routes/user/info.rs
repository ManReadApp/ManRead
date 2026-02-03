use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{
            gender::Gender,
            role::{Permission, Role},
        },
        manga::tag::Tag,
    },
    req::IdRequest,
};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiResult,
    models::{tag::TagDBService, user::UserDBService},
    services::achievement::Achievement,
};

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct User {
    id: String,
    pub names: Vec<String>,
    pub role: Role,
    pub tags: Vec<Tag>,
    pub achievements: Vec<Achievement>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub links: Vec<String>,
    pub thumb_ext: Option<String>,
    pub icon_ext: Option<String>,
    pub gender: Gender,
}

#[api_operation(
    tag = "user",
    summary = "Displays info about the user",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn exec(
    Json(data): Json<IdRequest>,
    user_service: Data<UserDBService>,
    tag_service: Data<TagDBService>,
) -> ApiResult<CreatedJson<User>> {
    let user = user_service.info(&data.id).await?;
    Ok(CreatedJson(User {
        id: user.id.id().to_string(),
        names: user.data.names,
        role: Role::try_from(user.data.role).unwrap(),
        tags: tag_service
            .get_tags(user.data.tags.into_iter().map(|v| v.thing.id().to_string()))
            .await?,
        achievements: user.data.achievements,
        bio: user.data.bio,
        location: user.data.location,
        links: user.data.links,
        thumb_ext: user.data.thumb_ext,
        icon_ext: user.data.icon_ext,
        gender: Gender::from(user.data.gender as usize),
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/info").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
