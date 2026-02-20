use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{Character, CreateCharacterRequest, IdRequest, SearchRequest},
    Permission,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{actions::character::CharacterActions, error::ApiResult};

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/character")
        .service(
            apistos::web::resource("/create").route(
                apistos::web::put()
                    .to(create)
                    .guard(AuthorityGuard::new(Permission::Create)),
            ),
        )
        .service(
            apistos::web::resource("/info").route(
                apistos::web::post()
                    .to(info)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
        .service(
            apistos::web::resource("/search").route(
                apistos::web::post()
                    .to(search)
                    .guard(AuthorityGuard::new(Permission::Read)),
            ),
        )
}

#[api_operation(
    tag = "character",
    summary = "Creates a character",
    description = r###"Returns the created character id"###
)]
pub(crate) async fn create(
    Json(data): Json<CreateCharacterRequest>,
    character_service: Data<CharacterActions>,
) -> ApiResult<CreatedJson<IdRequest>> {
    let id = character_service.create(data).await?;
    Ok(CreatedJson(IdRequest { id }))
}

#[api_operation(
    tag = "character",
    summary = "Returns info about a character",
    description = r###""###
)]
pub(crate) async fn info(
    Json(data): Json<IdRequest>,
    character_service: Data<CharacterActions>,
) -> ApiResult<Json<Character>> {
    character_service.info(&data.id).await.map(Json)
}

#[api_operation(
    tag = "character",
    summary = "Searches characters",
    description = r###""###
)]
pub(crate) async fn search(
    Json(data): Json<SearchRequest>,
    character_service: Data<CharacterActions>,
) -> ApiResult<Json<Vec<Character>>> {
    character_service.search(data).await.map(Json)
}
