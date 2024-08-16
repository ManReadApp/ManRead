use crate::errors::ApiResult;
use crate::services::db::manga::{Manga, MangaDBService};
use crate::services::db::manga_kind::MangaKindDBService;
use crate::services::db::tag::TagDBService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::protect;
use api_structure::models::auth::jwt::Claim;
use api_structure::models::manga::search::Array;
use api_structure::models::manga::status::Status;
use api_structure::req::manga::search::SearchRequest;
use api_structure::resp::manga::home::HomeResponse;
use api_structure::resp::manga::search::SearchResponse;
use rand::Rng;
use surrealdb_extras::RecordData;

#[post("/home")]
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
pub async fn home(
    manga: Data<MangaDBService>,
    tags: Data<TagDBService>,
    user: ReqData<Claim>,
    user_service: Data<UserDBService>,
    kind_service: Data<MangaKindDBService>,
    tag_service: Data<TagDBService>,
) -> ApiResult<Json<HomeResponse>> {
    let generate = |order, desc, query| {
        let query = match query {
            None => Array {
                or_post: None,
                or: false,
                items: vec![],
            },
            Some(v) => v,
        };
        SearchRequest {
            order,
            desc,
            limit: 20,
            page: 1,
            query,
        }
    };
    //let trending = generate(Order::Popularity, true, None);
    let newest = generate(Order::Created.to_string(), true, None);
    //let reading = generate(Order::LastRead, true, None);
    // let favorites = generate(
    // Order::Alphabetical,
    // false,
    // Some(ItemOrArray::Item(Item {
    //     not: false,
    //     data: ItemData::enum_("Favorites"),
    // })),
    // );
    let latest_updates = generate(Order::Updated.to_string(), true, None);
    let random = generate(Order::Random.to_string(), false, None);
    Ok(Json(HomeResponse {
        trending: vec![], //format(manga.search(trending, &user.id).await?, &tags).await,
        newest: format(
            manga
                .search(newest, &user.id, &user_service, &kind_service, &tag_service)
                .await?,
            &tags,
        )
        .await?,
        latest_updates: format(
            manga
                .search(
                    latest_updates,
                    &user.id,
                    &user_service,
                    &kind_service,
                    &tag_service,
                )
                .await?,
            &tags,
        )
        .await?,
        favorites: vec![], //format(manga.search(favorites, &user.id).await?, &tags,  &user_service, &kind_service, &tag_service).await,
        reading: vec![],
        // reading: format(manga.search(reading, &user.id).await?, &tags,  &user_service, &kind_service, &tag_service).await?,
        random: format(
            manga
                .search(random, &user.id, &user_service, &kind_service, &tag_service)
                .await?,
            &tags,
        )
        .await?,
    }))
}

pub async fn format(
    data: Vec<RecordData<Manga>>,
    tags: &Data<TagDBService>,
) -> ApiResult<Vec<SearchResponse>> {
    let mut result = vec![];
    for v in data {
        let mut t: Vec<String> = vec![];
        for tag in v.data.tags {
            t.push(tags.get_tag(&tag.thing.id().to_string()).await.unwrap().tag)
        }
        let number = rand::thread_rng().gen_range(0..v.data.covers.len());
        result.push(SearchResponse {
            manga_id: v.id.id().to_string(),
            titles: v.data.titles,
            tags: t,
            status: Status::try_from(v.data.status)?,
            ext: v.data.covers.get(number).unwrap().clone(),
            number: number as u32,
        })
    }
    Ok(result)
}
