use bytes::Bytes;
use futures::{pin_mut, stream, SinkExt};
use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_errors::PgEmbedError;
use pg_embed::pg_fetch::{PgFetchSettings, PG_V15};
use pg_embed::postgres::PgSettings;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::read_to_string;
use std::path::Path;

use tokio_postgres::{CopyInSink, NoTls};

async fn init_postgres(path: &Path, port: u16) -> Result<tokio_postgres::Client, PgEmbedError> {
    //https://huggingface.co/datasets/GriddleDean/mangaupdates/resolve/main/postgres.sql?download=true
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
    .await
    .unwrap();
    pg.setup().await.unwrap();
    // start postgresql database
    pg.start_db().await.unwrap();

    pg.create_database("mangaupdates").await.unwrap();
    let (client, connection) = tokio_postgres::connect(
        "user=postgres password=password dbname=mangaupdates host=localhost port=5433",
        NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
            panic!("mangaupdates db is not working")
        }
        drop(pg);
    });

    pg_restore(&client, &path.join("external/mangaupdates.sql")).await;
    Ok(client)
}

async fn pg_restore(client: &tokio_postgres::Client, dump: &Path) {
    let mut builder = vec![];
    let mut queries = vec![];
    let mut copies = vec![];
    let mut copy = false;
    for line in read_to_string(dump).unwrap().lines() {
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
    client.batch_execute(&queries.join("\n")).await.unwrap();
    for copy in copies {
        if let Some((query, lines)) = copy.split_once("\n") {
            let mut stream = stream::iter(
                lines
                    .split("\n")
                    .map(|s| Bytes::from(format!("{s}\n")))
                    .map(Ok::<_, tokio_postgres::Error>),
            );
            let sink: CopyInSink<Bytes> = client.copy_in(query).await.unwrap();
            pin_mut!(sink);
            sink.send_all(&mut stream).await.unwrap();
            let _ = sink.finish().await.unwrap();
        }
    }
}

// #[tokio::test]
// async fn temp() {
//     let client = init_postgres(&PathBuf::from("tests"), 5433).await.unwrap();
//     sleep(Duration::from_secs(10)).await;
//     let tags = client
//         .query("SELECT * FROM public.tags;", &vec![])
//         .await
//         .unwrap();
//     let mut tags = tags
//         .into_iter()
//         .map(|arr| arr.get("name"))
//         .collect::<Vec<String>>();
//     tags.sort();
//     File::create("temp.txt")
//         .unwrap()
//         .write_all(tags.join("\n").as_bytes())
//         .unwrap();
// }
#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub(crate) data: Array,
    pub(crate) order: Order,
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

        let sql = format!("SELECT * FROM info{};", query);
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum ItemOrArray {
    Item(Item),
    Array(Array),
}

impl ItemOrArray {
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
    pub(crate) items: Vec<ItemOrArray>,
}

impl Array {
    fn to_string(&self) -> Option<String> {
        let arr: Vec<_> = self.items.iter().filter_map(|v| v.to_string()).collect();
        if arr.is_empty() {
            return None;
        }
        let v = arr.join(match self.or {
            true => " or ",
            false => " and ",
        });
        if arr.len() == 1 {
            return Some(v);
        }
        Some(format!("({})", v))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub(crate) not: bool,
    pub(crate) data: ItemData,
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
                    "typ {}= (SELECT id FROM mtypes WHERE lower(name) = '{}' LIMIT 1)",
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
                        "(SELECT id FROM genres WHERE lower(name) = '{}')",
                        q.to_lowercase()
                    ),
                    IdOrValue::Id(id) => id.to_string(),
                }
            ),
            ItemData::Tag(v) => match v {
                IdOrValue::Value(v) => format!(
                    "tags && Array(SELECT id FROM tags WHERE lower(name) = '{}')",
                    v.to_lowercase()
                ),
                IdOrValue::Id(id) => format!("{} = ANY(tags)", id),
            },
            ItemData::Licensed(l) => format!("licensed {}= {}", n2, l),
            ItemData::Completed(c) => format!("completed {}= {}", n2, c),
            ItemData::Author(a) => match a {
                IdOrValue::Value(v) => format!(
                    "author && Array(SELECT id FROM ppl WHERE lower(name) = '{}')",
                    save_sql_str(&v.to_lowercase())
                ),
                IdOrValue::Id(id) => format!("{} = ANY(author)", id),
            },
            ItemData::Artist(a) => match a {
                IdOrValue::Value(v) => format!(
                    "artist && Array(SELECT id FROM ppl WHERE lower(name) = '{}')",
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
                        "{} && Array(SELECT id FROM ppl WHERE lower(name) = '{}')",
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
