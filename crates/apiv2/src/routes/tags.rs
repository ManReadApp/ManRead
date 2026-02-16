use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    v1::{SearchRequest, Tag},
    Permission,
};
use apistos::{
    api_operation,
    web::{scope, Scope},
};

use crate::{actions::tags::TagActions, error::ApiResult};

pub fn register() -> Scope {
    scope("/tags").service(
        apistos::web::resource("/search").route(
            apistos::web::post()
                .to(search)
                .guard(AuthorityGuard::new(Permission::Read)),
        ),
    )
}

#[api_operation(
    tag = "kind",
    summary = "Lists all manga kinds",
    description = r###""###
)]
pub(crate) async fn search(
    Json(data): Json<SearchRequest>,
    tags: Data<TagActions>,
) -> ApiResult<Json<Vec<Tag>>> {
    tags.search(&data.query).await.map(Json)
}
