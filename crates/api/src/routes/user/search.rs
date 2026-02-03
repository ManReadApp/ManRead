use actix_web::web::{Data, Json};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{gender::Gender, role::Permission},
    req::auth::UserSearchRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::user::UserDBService};

use super::list::SimpleUser;

fn reorder(names: &mut Vec<String>, query: &str) {
    names.sort_by_key(|name| {
        if name.starts_with(query) {
            0
        } else if name.contains(query) {
            1
        } else {
            2
        }
    });
}

#[api_operation(
    tag = "user",
    summary = "Searches the users",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn exec(
    Json(data): Json<UserSearchRequest>,
    user_service: Data<UserDBService>,
) -> ApiResult<CreatedJson<Vec<SimpleUser>>> {
    let items = user_service
        .search(&data.query, data.page, data.limit)
        .await?;
    let (mut found, not_found): (Vec<_>, Vec<_>) = items
        .into_iter()
        .map(|mut v| {
            reorder(&mut v.data.names, &data.query);
            v
        })
        .partition(|v| {
            v.data
                .names
                .get(0)
                .map(|v| v.to_lowercase().starts_with(&data.query.to_lowercase()))
                .unwrap_or(false)
        });
    found.extend(not_found);
    Ok(CreatedJson(
        found
            .into_iter()
            .map(|v| SimpleUser {
                id: v.id.id().to_string(),
                names: v.data.names,
                icon_ext: v.data.icon_ext,
                gender: Gender::from(v.data.gender as usize),
            })
            .collect::<Vec<_>>(),
    ))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/search").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
