use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{jwt::Claim, role::Permission},
        manga::status::Status,
    },
    req::manga::search::SearchRequest,
    resp::manga::search::SearchResponse,
};
use apistos::{actix::CreatedJson, api_operation, ApiComponent};
use schemars::JsonSchema;
use serde::Serialize;
use surrealdb_extras::{RecordData, RecordIdType, SurrealTableInfo as _};

use crate::{
    error::ApiResult,
    models::{
        manga::{Manga, MangaDBService},
        tag::TagDBService,
        user::User,
    },
};
use rand::thread_rng;
use rand::{rngs::ThreadRng, seq::IteratorRandom};

pub async fn convert_to_search_response(
    v: RecordData<Manga>,
    tag_service: &Data<TagDBService>,
    rng: &mut ThreadRng,
) -> ApiResult<SearchResponse> {
    let (number, ext) = v
        .data
        .covers
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .choose(rng)
        .unwrap();
    let tags = tag_service
        .get_tags(v.data.tags.into_iter().map(|v| v.thing.id().to_string()))
        .await?;
    Ok(SearchResponse {
        manga_id: v.id.id().to_string(),
        titles: v.data.titles,
        tags,
        status: Status::try_from(v.data.status).unwrap(),
        ext,
        number: number as u32,
    })
}

#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct SearchResponse_ {
    items: Vec<SearchResponse>,
    max: u64,
}

#[api_operation(tag = "manga", summary = "Search for manga", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<SearchRequest>,
    search_service: Data<MangaDBService>,
    tag_service: Data<TagDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<SearchResponse_>> {
    let (max, search) = search_service
        .search(
            data,
            RecordIdType::from((User::name(), user.id.as_str())),
            true,
        )
        .await?;
    let mut rng = thread_rng();

    let mut resp: Vec<SearchResponse> = Vec::with_capacity(search.len());
    for v in search {
        resp.push(convert_to_search_response(v, &tag_service, &mut rng).await?);
    }
    Ok(CreatedJson(SearchResponse_ { items: resp, max }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/search").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
