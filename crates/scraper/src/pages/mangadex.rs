use api_structure::error::{ApiErr, ApiErrorType};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
pub const UA: &str = "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:12.0) Gecko/20100101 Firefox/12.0";

use crate::ScrapeError;

pub async fn get_data(
    client: &Client,
    url: &str,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    let uuid = extract_uuid(url)?;
    let url = format!("https://api.mangadex.org/manga/{uuid}?includes[]=artist&includes[]=author&includes[]=cover_art");
    let json: Root = client
        .get(url)
        .header(USER_AGENT, UA)
        .send()
        .await?
        .json()
        .await?;
    let mut out = HashMap::new();
    let alt: HashMap<_, _> = json
        .data
        .attributes
        .alt_titles
        .into_iter()
        .flat_map(|v| v.into_iter())
        .collect();
    for d in json.data.relationships {
        if let Some(attr) = d.attributes {
            let v = out
                .entry(d.type_field)
                .or_insert(ItemOrArray::Array(vec![]));
            if let ItemOrArray::Array(v) = v {
                v.push(attr.to_string())
            }
        }
    }
    for tag in json.data.attributes.tags {
        let v = tag
            .attributes
            .name
            .get("en")
            .or_else(|| tag.attributes.name.iter().next().map(|v| v.1));
        if let Some(v) = v {
            let entry = out
                .entry(tag.attributes.group)
                .or_insert(ItemOrArray::Array(vec![]));
            if let ItemOrArray::Array(arr) = entry {
                arr.push(v.to_string());
            }
        }
    }
    out.insert("type".to_string(), ItemOrArray::Item(json.data.type_field));
    out.insert(
        "title".to_string(),
        ItemOrArray::Map(json.data.attributes.title),
    );
    out.insert("titles".to_string(), ItemOrArray::Map(alt));
    out.insert(
        "descriptions".to_string(),
        ItemOrArray::Map(json.data.attributes.description),
    );
    out.insert(
        "links".to_string(),
        ItemOrArray::Map(json.data.attributes.links),
    );
    out.insert(
        "original_language".to_string(),
        ItemOrArray::Item(json.data.attributes.original_language),
    );
    out.insert(
        "status".to_string(),
        ItemOrArray::Item(json.data.attributes.status),
    );
    out.insert(
        "year".to_string(),
        ItemOrArray::Item(json.data.attributes.year.to_string()),
    );
    out.insert(
        "content_rating".to_string(),
        ItemOrArray::Item(json.data.attributes.content_rating),
    );
    out.insert(
        "state".to_string(),
        ItemOrArray::Item(json.data.attributes.state),
    );
    out.insert(
        "target_audience".to_string(),
        ItemOrArray::Item(json.data.attributes.publication_demographic),
    );

    Ok(out)
}

pub fn extract_uuid(url: &str) -> Result<String, ScrapeError> {
    let url = url
        .replace("https://", "")
        .replace("http://", "")
        .replace("mangadex.org/title/", "");
    let url = url.split('/').filter(|x| x != &"").collect::<Vec<_>>();
    if !url.is_empty() {
        return Ok(url.first().unwrap().to_string());
    }
    Err(ScrapeError(ApiErr {
        message: Some("Couldnt extract id".to_string()),
        cause: None,
        err_type: ApiErrorType::ScrapeErrorParseError,
    }))
}

use crate::downloader::download;
use crate::services::metadata::ItemOrArray;
use api_structure::models::manga::external_search::ExternalSearchData;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Root {
    pub result: String,
    pub response: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Attributes,
    pub relationships: Vec<Relationship>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub title: HashMap<String, String>,
    pub alt_titles: Vec<HashMap<String, String>>,
    pub description: HashMap<String, String>,
    pub is_locked: bool,
    pub links: HashMap<String, String>,
    pub original_language: String,
    pub last_volume: String,
    pub last_chapter: String,
    pub publication_demographic: String,
    pub status: String,
    pub year: i64,
    pub content_rating: String,
    pub tags: Vec<Tag>,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: TagAttributes,
}

#[derive(Serialize, Deserialize)]
pub struct TagAttributes {
    pub name: HashMap<String, String>,
    pub group: String,
}

#[derive(Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Option<Attributes3>,
}

#[derive(Serialize, Deserialize)]
pub struct Attributes3 {
    pub name: Option<String>,
    pub locale: Option<String>,
    pub biography: Option<HashMap<String, String>>,
    version: Option<Value>,
    #[serde(rename = "createdAt")]
    created_at: Option<Value>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<Value>,
    volume: Option<Value>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl Display for Attributes3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = format!(
            "{} from {}\n",
            self.name.as_ref().map(|c| c.as_str()).unwrap_or_default(),
            self.locale.as_ref().map(|c| c.as_str()).unwrap_or("unkown")
        );
        if let Some(v) = &self.biography {
            str = format!(
                "{str}\n{}",
                v.iter()
                    .map(|(key, value)| format!("{key}: {value}\n"))
                    .collect::<String>()
            );
        }
        write!(
            f,
            "{}\n{}",
            str,
            self.other
                .iter()
                .filter(|v| !v.1.is_null())
                .map(|(key, value)| format!("{key}: {}\n", value.to_string()))
                .collect::<String>()
        )
    }
}

pub async fn search(
    client: &Client,
    query: ExternalSearchData,
) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
    let (query, page) = query.get_query();
    let limit = 25;
    let offset = (page - 1) * limit;
    let text = download(client.get(format!("https://api.mangadex.org/manga?limit={limit}&offset={offset}&includes[]=cover_art&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica&title={query}&includedTagsMode=AND&excludedTagsMode=OR")).header(USER_AGENT, UA), false).await?;
    let json: Root1 = serde_json::from_str(&text)?;
    Ok(json
        .data
        .into_iter()
        .map(|manga| ScrapeSearchResponse {
            title: manga
                .attributes
                .title
                .get("en")
                .map(|v| v.to_string())
                .unwrap_or(
                    manga
                        .attributes
                        .title
                        .iter()
                        .next()
                        .map(|v| v.1.to_string())
                        .unwrap_or_default(),
                ),
            url: format!("https://mangadex.org/title/{}", manga.id),
            cover: manga
                .relationships
                .iter()
                .find_map(|v| match v {
                    Relationship1::Data(v) => match v.r#type == "cover_art" {
                        true => Some(format!(
                            "https://mangadex.org/covers/{}/{}",
                            manga.id, v.attributes.file_name
                        )),
                        false => None,
                    },
                    Relationship1::Other(_) => None,
                })
                .unwrap_or_default(),
            r#type: None,
            status: Some(manga.attributes.status),
        })
        .collect())
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum Relationship1 {
    Data(RelationshipStruct),
    Other(Value),
}

#[derive(Serialize, Deserialize)]
struct Attributes1 {
    #[serde(rename = "fileName")]
    pub file_name: String,
}
#[derive(Serialize, Deserialize)]
struct Attributes2 {
    pub title: HashMap<String, String>,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
struct RelationshipStruct {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub attributes: Attributes1,
}

#[derive(Serialize, Deserialize)]
struct Manga {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub attributes: Attributes2,
    relationships: Vec<Relationship1>,
}

#[derive(Serialize, Deserialize)]
struct Root1 {
    pub data: Vec<Manga>,
}
