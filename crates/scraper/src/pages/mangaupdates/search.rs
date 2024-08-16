use crate::{find_workspace_root, ScrapeError};
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use bytes::Bytes;
use futures::{pin_mut, stream, SinkExt};
use log::warn;
use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PgFetchSettings, PG_V15};
use pg_embed::postgres::PgSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr::addr_of;
use std::sync::{Arc, Once};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use tokio_postgres::types::{FromSql, Kind, Type};
use tokio_postgres::{Client, CopyInSink, NoTls};

async fn init_postgres(path: &Path, port: u16) -> Result<Client, ScrapeError> {
    let mut pg = pg_embed::postgres::PgEmbed::new(
        PgSettings {
            database_dir: path.join("external/mangaupdates"),
            port,
            user: "postgres".to_string(),
            password: "password".to_string(),
            auth_method: PgAuthMethod::Plain,
            persistent: false,
            timeout: None,
            migration_dir: None,
        },
        PgFetchSettings {
            version: PG_V15,
            ..Default::default()
        },
    )
    .await?;
    pg.setup().await?;
    pg.start_db().await?;

    pg.create_database("mangaupdates").await?;
    let (client, connection) = tokio_postgres::connect(
        &format!("user=postgres password=password dbname=mangaupdates host=localhost port={port}"),
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            warn!("connection error: {}", e);
            panic!("mangaupdates db is not working")
        }
        drop(pg);
    });

    pg_restore(&client, &path.join("external/mangaupdates.sql")).await?;
    Ok(client)
}

async fn pg_restore(client: &Client, dump: &Path) -> Result<(), ScrapeError> {
    let mut builder = vec![];
    let mut queries = vec![];
    let mut copies = vec![];
    let mut copy = false;
    for line in read_to_string(dump)?.lines() {
        if line.starts_with("--") || line.is_empty() {
            continue;
        }
        builder.push(line.to_string());
        if line.ends_with("FROM stdin;") {
            copy = true;
        }
        if copy {
            if line == "\\." {
                builder.pop();
                let query = builder.join("\n");
                copies.push(query);
                builder = vec![];
                copy = false;
            }
        } else {
            if line.ends_with(";") && !line.ends_with("\\;") {
                let query = builder.join("\n");
                queries.push(query);
                builder = vec![];
            }
        }
    }
    client.batch_execute(&queries.join("\n")).await?;
    for copy in copies {
        if let Some((query, lines)) = copy.split_once("\n") {
            let mut stream = stream::iter(
                lines
                    .split("\n")
                    .map(|s| Bytes::from(format!("{s}\n")))
                    .map(Ok::<_, tokio_postgres::Error>),
            );
            let sink: CopyInSink<Bytes> = client.copy_in(query).await?;
            pin_mut!(sink);
            sink.send_all(&mut stream).await?;
            let _ = sink.finish().await?;
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Multi {
    Int(i64),
    Uint(u64),
    Float(f64),
    Bool(bool),
    Str(String),
    Duration(Duration),
    Arr(Vec<Multi>),
    None,
}

impl<'a> FromSql<'a> for Multi {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        match *ty.kind() {
            Kind::Array(ref member) => {
                return Ok(Self::Arr(Vec::from_sql(ty, raw)?));
            }
            _ => {}
        };
        match *ty {
            Type::TIMESTAMPTZ | Type::TIMESTAMP => Ok(Multi::Duration(
                SystemTime::from_sql(ty, raw)?.duration_since(UNIX_EPOCH)?,
            )),
            Type::INT2 => Ok(Multi::Int(i16::from_sql(ty, raw)? as i64)),
            Type::INT4 => Ok(Multi::Int(i32::from_sql(ty, raw)? as i64)),
            Type::OID => Ok(Multi::Uint(u32::from_sql(ty, raw)? as u64)),
            Type::INT8 => Ok(Multi::Int(i64::from_sql(ty, raw)?)),
            Type::CHAR => Ok(Multi::Int(i8::from_sql(ty, raw)? as i64)),
            Type::BOOL => Ok(Multi::Bool(bool::from_sql(ty, raw)?)),
            Type::FLOAT4 => Ok(Multi::Float(f32::from_sql(ty, raw)? as f64)),
            Type::FLOAT8 => Ok(Multi::Float(f64::from_sql(ty, raw)?)),
            Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::NAME | Type::UNKNOWN => {
                <&str as FromSql>::from_sql(ty, raw)
                    .map(ToString::to_string)
                    .map(Multi::Str)
            }
            ref ty
                if (ty.name() == "citext"
                    || ty.name() == "ltree"
                    || ty.name() == "lquery"
                    || ty.name() == "ltxtquery") =>
            {
                <&str as FromSql>::from_sql(ty, raw)
                    .map(ToString::to_string)
                    .map(Multi::Str)
            }
            _ => Err(format!("Unsupported type: {:?}", ty).into()),
        }
    }

    fn from_sql_null(_: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self::None)
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::TIMESTAMP | Type::TIMESTAMPTZ => true,
            Type::BOOL
            | Type::FLOAT8
            | Type::FLOAT4
            | Type::CHAR
            | Type::INT2
            | Type::INT4
            | Type::INT8
            | Type::OID => true,
            Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::NAME | Type::UNKNOWN => true,
            ref ty
                if (ty.name() == "citext"
                    || ty.name() == "ltree"
                    || ty.name() == "lquery"
                    || ty.name() == "ltxtquery") =>
            {
                true
            }
            _ => match ty.kind() {
                Kind::Array(_) => true,
                _ => false,
            },
        }
    }
}

static CLIENT: OnceCell<Client> = OnceCell::new();

pub async fn get_client() -> &'static Client {
    CLIENT
        .get_or_init(async {
            init_postgres(&find_workspace_root().unwrap().join("data"), 5437)
                .await
                .unwrap()
        })
        .await
}
pub async fn search(
    client: &Client,
    req: SearchRequest,
) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
    let query = req.to_string();
    let tags = client.query(&query, &vec![]).await?;
    let mut tags = tags
        .into_iter()
        .map(|arr| {
            arr.columns()
                .iter()
                .map(|v| (v.name().to_string(), arr.get(v.name())))
                .collect::<HashMap<String, Multi>>()
        })
        .map(|v| ScrapeSearchResponse {
            title: v
                .get("titles")
                .and_then(|v| match v {
                    Multi::Arr(v) => v.first().and_then(|v| match v {
                        Multi::Str(v) => Some(v.clone()),
                        _ => None,
                    }),
                    _ => None,
                })
                .unwrap_or_default(),
            url: format!(
                "https://www.mangaupdates.com/series/{}/",
                match v.get("url_key").expect("broken db") {
                    Multi::Str(v) => v,
                    _ => unreachable!(),
                }
            ),
            cover: v
                .get("image_name")
                .map(|v| match v {
                    Multi::Str(v) => format!("https://cdn.mangaupdates.com/image/{v}"),
                    _ => String::new(),
                })
                .unwrap_or_default(),
            r#type: v.get("typ").and_then(|v| match v {
                Multi::Int(v) => Some(v.to_string()),
                _ => None,
            }),
            status: v.get("status").and_then(|v| match v {
                Multi::Str(v) => Some(v.clone()),
                _ => None,
            }),
        })
        .collect::<Vec<ScrapeSearchResponse>>();
    let mut store = vec![];
    for v in &mut tags {
        if let Some(r#type) = v.r#type.clone() {
            match store.iter().find(|(a, _)| a == &r#type) {
                None => {
                    let typ: String = client
                        .query(
                            &format!("SELECT name FROM public.mtypes WHERE id = {}", r#type),
                            &vec![],
                        )
                        .await?
                        .first()
                        .expect("incomplete database")
                        .get("name");
                    v.r#type = Some(typ.clone());
                    store.push((r#type.clone(), typ));
                }
                Some((_, str)) => {
                    v.r#type = Some(str.to_string());
                }
            }
        }
    }
    Ok(tags)

    // // metadata
    // let mut v = tags.into_iter().map(|v|v.into_iter().filter(|(_, value)|{
    //     match value {
    //         Multi::None => false,
    //         Multi::Arr(v) => !v.is_empty(),
    //         _ => true
    //     }
    // }).collect::<HashMap<_, _>>()).collect::<Vec<_>>();
    // let mut out = vec![];
    // for item in v {
    //     let mut new = HashMap::new();
    //     for (key, value) in item {
    //         match key.as_str() {
    //             "publication_publisher" | "tags_uploader" | "tags_downvotes" | "tags_upvotes" => {}
    //             "typ" => {
    //                 let id = match value {
    //                     Multi::Int(v) => v,
    //                     _ => unreachable!()
    //                 };
    //                 let result = client
    //                     .query(&format!("SELECT name FROM public.mtypes WHERE id = {id}"), &vec![])
    //                     .await?;
    //                 let first: String = result.first().expect("incomplete db").get("name");
    //                 new.insert("type".to_string(), Multi::Str(first));
    //             },
    //             "publisher_english" | "publisher_original" | "author" | "artist" => {
    //                 let mut out = vec![];
    //                 match value {
    //                     Multi::Arr(v) => {
    //                         for v in v {
    //                             let id = match v {
    //                                 Multi::Int(v) => v,
    //                                 _ => unreachable!()
    //                             };
    //                             let result = client
    //                                 .query(&format!("SELECT name FROM public.ppl WHERE id = {id}"), &vec![])
    //                                 .await?;
    //                             let first: String = result.first().expect("incomplete db").get("name");
    //                             out.push(Multi::Str(first));
    //                         }
    //                     }
    //                     _ => unreachable!()
    //                 }
    //             },
    //             "tags" => {
    //                 let mut out = vec![];
    //                 match value {
    //                     Multi::Arr(v) => {
    //                         for v in v {
    //                             let id = match v {
    //                                 Multi::Int(v) => v,
    //                                 _ => unreachable!()
    //                             };
    //                             let result = client
    //                                 .query(&format!("SELECT name FROM public.tags WHERE id = {id}"), &vec![])
    //                                 .await?;
    //                             let first: String = result.first().expect("incomplete db").get("name");
    //                             out.push(Multi::Str(first));
    //                         }
    //                     }
    //                     _ => unreachable!()
    //                 }
    //                 new.insert(key, Multi::Arr(out));
    //             }
    //             "genres" => {
    //                 let mut out = vec![];
    //                 match value {
    //                     Multi::Arr(v) => {
    //                         for v in v {
    //                             let id = match v {
    //                                 Multi::Int(v) => v,
    //                                 _ => unreachable!()
    //                             };
    //                             let result = client
    //                                 .query(&format!("SELECT name FROM public.genres WHERE id = {id}"), &vec![])
    //                                 .await?;
    //                             let first: String = result.first().expect("incomplete db").get("name");
    //                             out.push(Multi::Str(first));
    //                         }
    //                     }
    //                     _ => unreachable!()
    //                 }
    //             }
    //             _ => {
    //                 new.insert(key, value);
    //             }
    //         }
    //     }
    //     out.push(new);
    // }
}
#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub(crate) data: Array,
    pub(crate) order: Order,
    pub limit: Limit,
}

#[derive(Serialize, Deserialize)]
pub struct Limit {
    pub size: u64,
    pub page: u64,
}

#[derive(Serialize, Deserialize)]
pub struct FilterRequest {
    filter: Vec<String>,
    name: String,
}

impl Display for SearchRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let query = match self.data.to_string() {
            Some(v) => format!(" WHERE {}", v),
            None => String::new(),
        };

        let order = self.order.to_string();

        let sql = format!(
            "SELECT * FROM public.info{} {} LIMIT {} OFFSET {};",
            query,
            order,
            self.limit.size,
            (self.limit.page - 1) * self.limit.size
        );
        write!(f, "{}", sql)
    }
}

fn save_sql_str(str: &str) -> String {
    str.replace('\'', "''")
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[derive(Serialize, Deserialize)]
pub struct Order {
    pub(crate) desc: bool,
    pub(crate) kind: OrderKind,
}

#[derive(Serialize, Deserialize)]
pub enum OrderKind {
    Id,
    PrivateId,
    Title,
    LastUpdatedMU,
}

impl TryFrom<String> for OrderKind {
    type Error = ScrapeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "id" => Ok(Self::Id),
            "private_id" => Ok(Self::PrivateId),
            "title" => Ok(Self::Title),
            "last_updated_mu" => Ok(Self::LastUpdatedMU),
            _ => Err(ScrapeError::input_error("not valid order")),
        }
    }
}

impl Display for OrderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Id => "id",
                OrderKind::PrivateId => "private_id",
                OrderKind::Title => "title",
                OrderKind::LastUpdatedMU => "last_updated_mu",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum ItemOrArray {
    Item(Item),
    Array(Array),
}

impl ItemOrArray {
    fn or(&self) -> Option<bool> {
        match self {
            ItemOrArray::Item(v) => v.or,
            ItemOrArray::Array(v) => v.or_post,
        }
    }
    fn to_string(&self) -> Option<String> {
        match self {
            ItemOrArray::Item(v) => Some(v.to_string()),
            ItemOrArray::Array(v) => v.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Array {
    pub(crate) or: bool,
    pub or_post: Option<bool>,
    pub(crate) items: Vec<ItemOrArray>,
}

impl TryFrom<api_structure::models::manga::search::Array> for Array {
    type Error = ScrapeError;

    fn try_from(value: api_structure::models::manga::search::Array) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Array {
    fn to_string(&self) -> Option<String> {
        let arr: Vec<_> = self
            .items
            .iter()
            .filter_map(|v| v.to_string().map(|va| (va, v.or())))
            .collect();
        if arr.is_empty() {
            return None;
        }
        let generate = |b| match b {
            true => "or",
            false => "and",
        };
        let default = generate(self.or);
        let v = arr
            .into_iter()
            .map(|(a, b)| format!("{a} {} ", b.map(|v| generate(v)).unwrap_or(default)))
            .collect::<String>();
        Some(format!("({})", v))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub(crate) not: bool,
    pub(crate) data: ItemData,
    or: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ItemData {
    Id(i32),
    Pid(i32),
    PublicId(i64),
    ForumId(i64),
    Key(String),
    Title(String),
    Description(String),
    Type(IdOrValue),
    Year { eq: bool, bigger: bool, value: i32 },
    LatestChapter { eq: bool, bigger: bool, value: i32 },
    Tag(IdOrValue),
    Genre(IdOrValue),
    Licensed(bool),
    Completed(bool),
    Artist(IdOrValue),
    Author(IdOrValue),
    Publisher { value: IdOrValue, eng: bool },
    Rating { eq: bool, bigger: bool, rating: f32 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IdOrValue {
    Value(String),
    Id(i32),
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let d = match self.desc {
            true => "DESC",
            false => "ASC",
        };
        write!(f, "{} {}", self.kind, d)
    }
}

impl Display for OrderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Id => "id",
                OrderKind::PrivateId => "private_id",
                OrderKind::Title => "title",
                OrderKind::LastUpdatedMU => todo!(),
            }
        )
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = match self.not {
            true => "not ",
            false => "",
        };
        let n2 = match self.not {
            true => "!",
            false => "",
        };
        let str = match &self.data {
            ItemData::Id(id) => format!("id {}= {}", n2, id),
            ItemData::Pid(id) => format!("private_id {}= {}", n2, id),
            ItemData::PublicId(id) => format!("public_id {}= {}", n2, id),
            ItemData::ForumId(id) => format!("forum_id {}= {}", n2, id),
            ItemData::Key(key) => format!("url_key {}= {}", n2, key),
            ItemData::Title(title) => format!(
                "{}EXISTS(SELECT 1 FROM unnest(titles) as element WHERE lower(element) ILIKE '%{}%')",
                n,
                save_sql_str(&title.to_lowercase())
            ),
            ItemData::Description(description) => {
                format!("lower(description) {}ILIKE '%{}%'", n, save_sql_str(&description.to_lowercase()))
            }
            ItemData::Type(t) => match t {
                IdOrValue::Value(q) => format!(
                    "typ {}= (SELECT id FROM public.mtypes WHERE lower(name) = '{}' LIMIT 1)",
                    n2,
                    save_sql_str(&q.to_lowercase())
                ),
                IdOrValue::Id(id) => format!("typ {}= {}", n2, id),
            },
            ItemData::Year { eq, bigger, value } => {
                format!("year {}", year(self.not, *bigger, *eq, value))
            }
            ItemData::LatestChapter { eq, bigger, value } => {
                format!("latest_chapter {}", year(self.not, *bigger, *eq, value))
            }
            ItemData::Rating { eq, bigger, rating } => {
                format!("bayesian_rating {}", year(self.not, *bigger, *eq, rating))
            }
            ItemData::Genre(v) => format!(
                "{} = ANY(genres)",
                match v {
                    IdOrValue::Value(q) => format!(
                        "(SELECT id FROM public.genres WHERE lower(name) = '{}')",
                        q.to_lowercase()
                    ),
                    IdOrValue::Id(id) => id.to_string(),
                }
            ),
            ItemData::Tag(v) => match v {
                IdOrValue::Value(v) => format!(
                    "tags && Array(SELECT id FROM public.tags WHERE lower(name) = '{}')",
                    v.to_lowercase()
                ),
                IdOrValue::Id(id) => format!("{} = ANY(tags)", id),
            },
            ItemData::Licensed(l) => format!("licensed {}= {}", n2, l),
            ItemData::Completed(c) => format!("completed {}= {}", n2, c),
            ItemData::Author(a) => match a {
                IdOrValue::Value(v) => format!(
                    "author && Array(SELECT id FROM public.ppl WHERE lower(name) = '{}')",
                    save_sql_str(&v.to_lowercase())
                ),
                IdOrValue::Id(id) => format!("{} = ANY(author)", id),
            },
            ItemData::Artist(a) => match a {
                IdOrValue::Value(v) => format!(
                    "artist && Array(SELECT id FROM public.ppl WHERE lower(name) = '{}')",
                    save_sql_str(&v.to_lowercase())
                ),
                IdOrValue::Id(id) => format!("{} = ANY(artist)", id),
            },
            ItemData::Publisher { value, eng } => {
                let field = match eng {
                    true => "publisher_english",
                    false => "publisher_original"
                };
                match value {
                    IdOrValue::Value(v) => format!(
                        "{} && Array(SELECT id FROM public.ppl WHERE lower(name) = '{}')",
                        field,
                        save_sql_str(&v.to_lowercase())
                    ),
                    IdOrValue::Id(id) => format!("{} = ANY({})", id, field)
                }
            }
        };
        write!(f, "{}", str)
    }
}

fn year(not: bool, mut bigger: bool, mut eq: bool, number: impl Display) -> String {
    if not {
        eq = !eq;
        bigger = !bigger;
    }

    let eq = match eq {
        true => "=",
        false => "",
    };
    let sb = match bigger {
        true => ">",
        false => "<",
    };
    format!("{}{} {}", sb, eq, number)
}
