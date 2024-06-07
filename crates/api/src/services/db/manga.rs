use crate::errors::{ApiError, ApiResult};
use crate::services::db::chapter::Chapter;
use crate::services::db::manga_kind::{Kind, MangaKindDBService};
use crate::services::db::tag::{Tag, TagDBService};
use crate::services::db::user::{User, UserDBService};
use crate::services::db::version::Version;
use actix_web::web::Data;
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::search::{ItemData, ItemOrArray, ItemValue, Order, SearchRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use surrealdb_extras::{
    RecordData, SurrealSelect, SurrealSelectInfo, SurrealTable, SurrealTableInfo, ThingFunc,
    ThingType,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("mangas")]
#[sql(["DEFINE EVENT manga_updated ON TABLE mangas WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct Manga {
    pub titles: HashMap<String, Vec<String>>,
    pub kind: ThingType<Kind>,
    pub description: Option<String>,
    pub tags: Vec<ThingType<Tag>>,
    pub status: u64,
    pub visibility: u64,
    pub uploader: ThingType<User>,
    pub artists: Vec<ThingType<User>>,
    pub authors: Vec<ThingType<User>>,
    pub covers: Vec<String>,
    pub chapters: Vec<ThingType<Chapter>>,
    pub sources: Vec<String>,
    pub relations: Vec<ThingType<Manga>>,
    pub scraper: Vec<ThingType<Version>>,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(SurrealSelect, Deserialize)]
pub struct MangaSearch {
    pub titles: HashMap<String, Vec<String>>,
    pub tags: Vec<ThingType<Tag>>,
}

impl Hash for Manga {
    fn hash<H: Hasher>(&self, _: &mut H) {
        unimplemented!()
    }
}

impl PartialEq<Self> for Manga {
    fn eq(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

impl Eq for Manga {}

pub struct MangaDBService {
    conn: Arc<Surreal<Db>>,
}

impl MangaDBService {
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self { conn }
    }

    pub async fn get(&self, id: &str) -> ApiResult<RecordData<Manga>> {
        let thing = ThingFunc::from((Manga::name(), id));
        Ok(thing.get(&*self.conn).await?.ok_or(ApiErr {
            message: Some("failed to find record".to_string()),
            cause: None,
            err_type: ApiErrorType::InternalError,
        })?)
    }

    pub async fn search(
        &self,
        search: SearchRequest,
        user_id: &str,
        user_service: &Data<UserDBService>,
        kind_service: &Data<MangaKindDBService>,
        tag_service: &Data<TagDBService>,
    ) -> ApiResult<Vec<RecordData<Manga>>> {
        let query = query_builder(
            search,
            &Manga::keys().join(","),
            user_id,
            user_service,
            kind_service,
            tag_service,
        )
        .await?;
        println!("{}", query);
        Ok(self.conn.query(query).await?.take(0)?)
    }
}

enum ItemDataDefined {
    Favorites,
    Reading,
    Title(String),
    Artist(String),
    Author(String),
    Uploader(String),
    Chapters(ItemValue),
    Uploaded(ItemValue),
    Kind(String),
    Source(String),
    Status(i64),
    Tag { sex: Option<u32>, value: String },
}

impl ItemDataDefined {
    async fn sql(
        &self,
        user: &str,
        not: bool,
        user_service: &Data<UserDBService>,
        kind_service: &Data<MangaKindDBService>,
        tag_service: &Data<TagDBService>,
    ) -> ApiResult<String> {
        let not2 = match not {
            true => "!",
            false => "",
        };

        let not1 = match not {
            true => "NOT ",
            false => "",
        };

        match self {
            ItemDataDefined::Favorites => Ok(format!(
                r#"count(SELECT id FROM scrape_list WHERE name = "Favorites" AND user = {user} mangas CONTAINS $after.id LIMIT 1) {not2}= 1"#
            )),
            ItemDataDefined::Reading => todo!(),
            ItemDataDefined::Title(title) => Ok(format!(
                "(array::flatten(object::values(titles)) *~ \"{title}\") {not2}= true"
            )),
            ItemDataDefined::Source(source) => {
                Ok(format!("(sources *~ \"{source}\") {not2}= true",))
            }
            ItemDataDefined::Artist(user) => Ok(format!(
                "{} {}IN artists",
                user_service.get_id(user, false).await?,
                not1
            )),
            ItemDataDefined::Author(user) => Ok(format!(
                "{} {}IN authors",
                user_service.get_id(user, false).await?,
                not1
            )),
            ItemDataDefined::Uploader(user) => Ok(format!(
                "uploader {}= {}",
                not2,
                user_service.get_id(user, false).await?
            )),
            ItemDataDefined::Chapters(ItemValue::CmpInt { bigger, eq, value }) => Ok(format!(
                "count(chapters) {} {}",
                display_eq(*bigger, *eq, not),
                value
            )),
            ItemDataDefined::Uploaded(ItemValue::CmpInt { bigger, eq, value }) => Ok(format!(
                "created {} {}",
                display_eq(*bigger, *eq, not),
                DateTime::<Utc>::from_timestamp_millis(*value).unwrap_or(DateTime::<Utc>::MIN_UTC)
            )),
            ItemDataDefined::Kind(v) => {
                Ok(format!("kind {}= {}", not2, kind_service.get_id(v).await?))
            }
            ItemDataDefined::Status(v) => Ok(format!("status {}= {}", not2, v)),
            ItemDataDefined::Tag { value, sex } => Ok(format!(
                "{} {} tags",
                tag_service.get_ids(sex, value).await?,
                match not {
                    true => "NONEINSIDE",
                    false => "ANYINSIDE",
                },
            )),
            ItemDataDefined::Chapters(_) => unreachable!(),
            ItemDataDefined::Uploaded(_) => unreachable!(),
        }
    }
}

fn display_eq(mut bigger: bool, mut eq: bool, not: bool) -> String {
    if not {
        bigger = !bigger;
        eq = !eq;
    }
    format!(
        "{}{}",
        match bigger {
            true => ">",
            false => "<",
        },
        match eq {
            true => "=",
            false => "",
        }
    )
}

impl TryFrom<ItemData> for ItemDataDefined {
    type Error = ApiError;

    fn try_from(value: ItemData) -> Result<Self, Self::Error> {
        let key = value.name.to_lowercase();
        let key = key.as_str();
        if key == "favorites" && matches!(value.value, ItemValue::None) {
            return Ok(ItemDataDefined::Favorites);
        } else if key == "reading" && matches!(value.value, ItemValue::None) {
            return Ok(ItemDataDefined::Reading);
        } else if key == "title" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Title(s));
            }
        } else if key == "artist" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Artist(s));
            }
        } else if key == "author" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Author(s));
            }
        } else if key == "uploader" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Uploader(s));
            }
        } else if key == "chapters" && matches!(value.value, ItemValue::CmpInt { .. }) {
            return Ok(ItemDataDefined::Chapters(value.value));
        } else if key == "uploaded" && matches!(value.value, ItemValue::CmpInt { .. }) {
            return Ok(ItemDataDefined::Uploaded(value.value));
        } else if key == "kind" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Kind(s));
            }
        } else if key == "source" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Source(s));
            }
        } else if key == "status" {
            if let ItemValue::Int(s) = value.value {
                return Ok(ItemDataDefined::Status(s));
            }
        } else if key == "tag" {
            if let ItemValue::String(s) = value.value {
                return Ok(ItemDataDefined::Tag {
                    sex: None,
                    value: s,
                });
            }
        }
        Err(ApiErr {
            message: Some("Couldnt find ItemData".to_string()),
            cause: Some(value.name),
            err_type: ApiErrorType::InvalidInput,
        }
        .into())
    }
}

#[async_recursion::async_recursion]
async fn to_sql(
    item: ItemOrArray,
    user_id: &str,
    user_service: &Data<UserDBService>,
    kind_service: &Data<MangaKindDBService>,
    tag_service: &Data<TagDBService>,
) -> ApiResult<String> {
    Ok(match item {
        ItemOrArray::Item(v) => {
            let item = ItemDataDefined::try_from(v.data)?;
            item.sql(user_id, v.not, user_service, kind_service, tag_service)
                .await?
        }
        ItemOrArray::Array(v) => {
            let mut data = vec![];
            for item in v.items {
                data.push(to_sql(item, user_id, user_service, kind_service, tag_service).await?)
            }
            let join = if v.or { "OR" } else { "AND" };
            data.join(&format!(" {} ", join))
        }
    })
}

async fn query_builder(
    r: SearchRequest,
    fields: &str,
    user_id: &str,
    user_service: &Data<UserDBService>,
    kind_service: &Data<MangaKindDBService>,
    tag_service: &Data<TagDBService>,
) -> ApiResult<String> {
    let asc = if r.desc { "DESC" } else { "ASC" };
    //TODO: list_count

    let (order, table) = if Order::LastRead != r.order {
        let order = match r.order {
            Order::Random => "ORDER BY RAND()".to_string(),
            _ => format!(
                "ORDER BY {} {}",
                match r.order {
                    Order::Created => "created",
                    Order::Alphabetical => "title",
                    Order::Updated => "updated",
                    Order::LastRead => unreachable!(),
                    Order::Popularity => "list_count",
                    Order::Random => unreachable!(),
                },
                asc
            ),
        };
        (order, Manga::name().to_string())
    } else {
        // let new_item = ItemOrArray::Item(Item::new(ItemData::enum_("Reading")));
        // r.query = match r.query {
        //     ItemOrArray::Item(v) => ItemOrArray::Array(Array {
        //         or: false,
        //         items: vec![ItemOrArray::Item(v), new_item],
        //     }),
        //     ItemOrArray::Array(mut v) => {
        //         v.items.push(new_item);
        //         v.or = false;
        //         ItemOrArray::Array(v)
        //     }
        // };
        ("".to_string(), reading(user_id))
    };
    let query = to_sql(r.query, user_id, user_service, kind_service, tag_service).await?;
    let limit = format!("LIMIT {} START {}", r.limit, (r.page - 1) * r.limit);
    let base = format!("SELECT {fields} FROM {table}");
    if query.is_empty() {
        Ok(format!("{base} {order} {limit}"))
    } else {
        Ok(format!("{base} WHERE {query} {order} {limit}"))
    }
}

//
// DEFINE FUNCTION fn::get_name($priority: array<string>, $map: object) {
//   FOR $prio IN $priority {
//     FOR $item IN object::entries($map) {
//       LET $key = array::at($item, 0);
//       LET $value = array::at($item, 1);
//       IF $key == $prio THEN
//         LET $value1 = array::first($value);
//         IF $value1 != NULL THEN
//           return $value1;
//         END
//       END
//     }
//   }
//
// LET $first = array::first(array::flatten(object::values($map)))
// IF $first != NULL THEN
//   return $first;
// END
// return "No title";
// }

fn last_read() -> &'static str {
    //f32, datetime
    "SELECT progress, updated as read_updated FROM user_progress WHERE user_progress.user = {} AND user_progress.manga = mangas.id ORDER BY user_progress.updated DESC LIMIT 1"
}

fn popularity() -> &'static str {
    // number => list_count
    "count(SELECT id FROM user_progress WHERE user_progress.manga = mangas.id)"
}

fn reading(user_id: &str) -> String {
    format!("(SELECT manga FROM (SELECT manga, time::max(updated) as max FROM user_progress WHERE user = {user_id} GROUP BY manga) ORDER BY max DESC)")
}
