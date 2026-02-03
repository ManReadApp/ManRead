use actix_web::web::{Data, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::{
        auth::{jwt::Claim, role::Permission},
        manga::search::{Array, Item, ItemData, ItemOrArray, ItemValue, Order},
    },
    req::manga::search::SearchRequest,
    resp::manga::{home::HomeResponse, search::SearchResponse},
};
use apistos::{actix::CreatedJson, api_operation};
use rand::thread_rng;
use surrealdb_extras::{RecordIdType, SurrealTableInfo as _};

use crate::{
    error::{ApiError, ApiResult},
    models::{manga::MangaDBService, tag::TagDBService, user::User},
};

use super::search::convert_to_search_response;

#[api_operation(
    tag = "manga",
    summary = "Gets all the info for the manga home",
    description = r###""###
)]
pub(crate) async fn exec(
    manga_service: Data<MangaDBService>,
    tag_service: Data<TagDBService>,
    user: ReqData<Claim>,
) -> ApiResult<CreatedJson<HomeResponse>> {
    let generate = |order: Order, desc, query| {
        let items = match query {
            None => vec![],
            Some(v) => vec![v],
        };
        SearchRequest {
            order: order.to_string(),
            desc,
            limit: 20,
            page: 1,
            query: Array {
                not: false,
                or_post: None,
                or: false,
                items,
            },
        }
    };

    let search = |req| async {
        let mut rng = thread_rng();

        let (_, v) = manga_service
            .search(
                req,
                RecordIdType::from((User::name(), user.id.as_str())),
                false,
            )
            .await?;

        let mut resp: Vec<SearchResponse> = Vec::with_capacity(v.len());
        for v in v {
            resp.push(convert_to_search_response(v, &tag_service, &mut rng).await?);
        }
        Ok::<_, ApiError>(resp)
    };
    let trending = generate(Order::Popularity, true, None);
    let newest = generate(Order::Created, true, None);
    let reading = generate(Order::LastRead, true, None);
    let favorites = generate(
        Order::Alphabetical,
        false,
        Some(ItemOrArray::Item(Item {
            not: false,
            or_post: None,
            data: ItemData {
                name: "list".to_owned(),
                value: ItemValue::String("favorites".to_owned()),
            },
        })),
    );
    let latest_updates = generate(Order::Updated, true, None);
    let random = generate(Order::Random, false, None);

    Ok(CreatedJson(HomeResponse {
        trending: vec![], //search(trending).await?,
        newest: search(newest).await?,
        latest_updates: search(latest_updates).await?,
        favorites: search(favorites).await?,
        reading: search(reading).await?,
        random: search(random).await?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/home").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
