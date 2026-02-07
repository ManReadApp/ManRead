pub mod req;

use std::{
    collections::{BTreeMap, HashMap},
    num::{ParseFloatError, ParseIntError},
    sync::Arc,
};

pub use api_structure::models::manga::external_search::ValidSearches;
use async_trait::async_trait;
#[cfg(feature = "json")]
use serde_json::Value;

#[derive(Clone)]
pub struct Functions {
    pub guess_episode: fn(&str) -> Result<f64, ScraperError>,
}

#[derive(Clone)]
pub struct ScrapeAccount {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum ScraperError {
    Reqwest(String),
    SerdeJson(String),
    InvalidChapterNum(String),
    Utf8(String),
    InvalidQuery,
    CookieNotFound,
    Unimplemented,
    InvalidUrl,
    Cloudflare,
    NoJsonFeature,
    AttrNotFound,
    NodeNotFound,
    ApiError { status: u16, message: String },
}

impl From<ParseFloatError> for ScraperError {
    fn from(e: ParseFloatError) -> Self {
        ScraperError::InvalidChapterNum(e.to_string())
    }
}

impl From<ParseIntError> for ScraperError {
    fn from(e: ParseIntError) -> Self {
        ScraperError::InvalidChapterNum(e.to_string())
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for ScraperError {
    fn from(e: serde_json::Error) -> Self {
        ScraperError::SerdeJson(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ScraperError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ScraperError::Utf8(e.to_string())
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ScraperError {
    fn from(e: reqwest::Error) -> Self {
        ScraperError::Reqwest(e.to_string())
    }
}

pub type ScraperResult<T> = Result<T, ScraperError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScrapedData {
    Str(String),
    Arr(Vec<ScrapedData>),
    Map2(Vec<(String, ScrapedData)>),
    Map(BTreeMap<String, ScrapedData>),
}

impl ScrapedData {
    pub fn flatten_vec(self) -> ScrapedData {
        match self {
            ScrapedData::Arr(vec) => {
                let mut flat = Vec::new();
                for item in vec {
                    match item.flatten_vec() {
                        ScrapedData::Arr(nested) => flat.extend(nested),
                        other => flat.push(other),
                    }
                }
                ScrapedData::Arr(flat)
            }
            ScrapedData::Map(map) => {
                let new_map = map
                    .into_iter()
                    .map(|(k, v)| (k, v.flatten_vec()))
                    .collect::<BTreeMap<_, _>>();
                ScrapedData::Map(new_map)
            }
            other => other,
        }
    }
}

#[cfg(feature = "json")]
impl From<ScrapedData> for Value {
    fn from(value: ScrapedData) -> Self {
        match value {
            ScrapedData::Str(s) => Value::String(s),
            ScrapedData::Arr(scraped_datas) => {
                Value::Array(scraped_datas.into_iter().map(|v| Value::from(v)).collect())
            }
            ScrapedData::Map(hash_map) => Value::Object(
                hash_map
                    .into_iter()
                    .map(|v| (v.0, Value::from(v.1)))
                    .collect(),
            ),
            ScrapedData::Map2(items) => Value::Array(
                items
                    .into_iter()
                    .map(|v| Value::Object(vec![(v.0, Value::from(v.1))].into_iter().collect()))
                    .collect(),
            ),
        }
    }
}

impl PartialOrd for ScrapedData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ScrapedData::Str(a), ScrapedData::Str(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}
impl Ord for ScrapedData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ScrapedData::Str(a), ScrapedData::Str(b)) => a.cmp(b),
            _ => panic!("Cannot compare {:?} with {:?}", self, other),
        }
    }
}

impl ScrapedData {
    pub fn get_str(&self) -> Option<String> {
        match self {
            ScrapedData::Str(v) => Some(v.clone()),
            ScrapedData::Arr(v) => match v.len() == 1 {
                true => v.first().unwrap().get_str(),
                false => None,
            },
            _ => None,
        }
    }
    pub fn new_array() -> Self {
        ScrapedData::Arr(vec![])
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            ScrapedData::Str(v) => Some(v.as_str()),
            ScrapedData::Arr(v) => match v.len() == 1 {
                true => v.first().unwrap().as_str(),
                false => None,
            },
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<Vec<ScrapedData>> {
        match self {
            ScrapedData::Arr(v) => Some(v.clone()),
            ScrapedData::Str(_) => Some(vec![self.clone()]),
            _ => None,
        }
    }
    pub fn as_map(&self) -> Option<BTreeMap<String, ScrapedData>> {
        match self {
            ScrapedData::Map(v) => Some(v.clone()),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "json", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ScrapedChapter {
    pub names: Vec<String>,
    pub chapter: f64,
    pub url: String,
    pub tags: Vec<String>,
}

impl ScrapedChapter {
    pub fn add_title(mut self, title: Option<String>) -> Self {
        if let Some(title) = title {
            self.names.push(title);
        }
        self
    }
}

pub trait Register: Send + Sync {
    /// Registers which processors are used
    fn get_used_processor_names(&self) -> Vec<&str>;
    /// Url belongs to site that iy scraped from
    fn url_matches(&self, url: &str) -> bool;
    /// content-type, Icon
    fn icon(&self) -> (String, Vec<u8>);
    /// returns the source of the icon
    fn icon_source(&self) -> Option<String>;
}

#[cfg_attr(
    feature = "openapi",
    derive(apistos::ApiComponent, schemars::JsonSchema)
)]
#[cfg_attr(feature = "json", derive(serde::Deserialize))]
pub enum SearchQuery {
    Simple(HashMap<String, Attribute>),
}

impl SearchQuery {
    pub fn validate(&self, AttributesDef(validator): AttributesDef) -> bool {
        match self {
            SearchQuery::Simple(hash_map) => {
                for (key, value) in hash_map {
                    let validator = match validator.get(key) {
                        Some(v) => v,
                        None => return false,
                    };

                    if !value.matches(&validator.value) {
                        return false;
                    }
                }
                if !validator
                    .iter()
                    .filter(|v| v.1.required)
                    .map(|v| v.0)
                    .all(|v| hash_map.contains_key(v))
                {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg_attr(feature = "json", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "openapi",
    derive(apistos::ApiComponent, schemars::JsonSchema)
)]
#[cfg_attr(feature = "json", serde(untagged))]
pub enum Attribute {
    Str(String),
    Int(i64),
    Bool(bool),
    Arr(Vec<Attribute>),
}

impl Attribute {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Attribute::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Attribute::Int(v) => Some(*v),
            _ => None,
        }
    }
    pub fn matches(&self, val: &AttributeValueDef) -> bool {
        match (self, val) {
            (Attribute::Str(_), AttributeValueDef::Str) => true,
            (Attribute::Int(_), AttributeValueDef::Int) => true,
            (Attribute::Bool(_), AttributeValueDef::Bool) => true,
            (Attribute::Arr(v), AttributeValueDef::Arr(items)) => {
                v.iter().all(|v| v.matches(items))
            }
            (Attribute::Str(v), AttributeValueDef::Enum(items)) => items.contains(v),
            _ => false,
        }
    }
}

pub struct AttributesDef(pub HashMap<String, AttributeDef>);
impl AttributesDef {
    pub fn expend(&mut self, items: impl IntoIterator<Item = (String, AttributeDef)>) {
        self.0.extend(items);
    }
}

impl Default for AttributesDef {
    fn default() -> Self {
        AttributesDef(
            [
                (
                    "query".to_owned(),
                    AttributeDef {
                        required: true,
                        value: AttributeValueDef::Str,
                    },
                ),
                (
                    "page".to_owned(),
                    AttributeDef {
                        required: true,
                        value: AttributeValueDef::Int,
                    },
                ),
            ]
            .into(),
        )
    }
}

pub struct AttributeDef {
    pub required: bool,
    pub value: AttributeValueDef,
}

pub enum AttributeValueDef {
    Str,
    Int,
    Bool,
    Arr(Box<AttributeValueDef>),
    Enum(Vec<String>),
}

impl SearchQuery {
    pub fn as_simple(&self) -> Option<&HashMap<String, Attribute>> {
        match self {
            SearchQuery::Simple(v) => Some(v),
        }
    }
}

#[derive(Clone)]
pub struct AdvadedSearchQuery {
    pub query: String,
    pub tags: Vec<String>,
    pub page: usize,
}

#[derive(Debug)]
#[cfg_attr(feature = "json", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "openapi",
    derive(apistos::ApiComponent, schemars::JsonSchema)
)]
pub struct ScrapedSearchResponse {
    pub title: String,
    pub url: String,
    pub cover: Option<String>,
    pub status: Option<String>,
    pub ty: Option<String>,
}

#[cfg_attr(feature = "json", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "openapi",
    derive(apistos::ApiComponent, schemars::JsonSchema)
)]
pub struct ExternalSearchResponse {
    pub items: Vec<ScrapedSearchResponse>,
    pub next_page: Option<usize>,
    pub prev_page: Option<usize>,
    pub last_page: Option<usize>,
    pub page: usize,
}

#[async_trait]
pub trait SearchScraper: Send + Sync {
    async fn search(&self, query: SearchQuery) -> ScraperResult<ExternalSearchResponse>;
    fn query(&self) -> ValidSearches;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(serde::Deserialize))]
pub enum Mode {
    Single,
    Multi,
    TextSingle,
    TextMulti,
}
#[async_trait]
pub trait ReaderScraper: Send + Sync {
    /// Returns true if the scraper can handle multiple chapters/manga metadata
    fn multi(&self, url: &str) -> Mode;
    async fn download_file(&self, url: &str) -> ScraperResult<Vec<u8>>;
    /// Returns urls to the pages
    async fn scrape_pages(&self, url: &str) -> ScraperResult<Vec<String>>;
    /// Returns array of chapters
    async fn scrape_chapters(&self, url: &str) -> ScraperResult<Vec<ScrapedChapter>>;
}

#[async_trait]
pub trait MetaDataScraper: Send + Sync {
    /// defaults to metadata from chapter if multi is false
    async fn scrape_metadata(
        &self,
        url: &str,
    ) -> Result<BTreeMap<String, ScrapedData>, ScraperError>;
}

/// Struct used to override/register scrapers
pub struct RegisterOverride {
    pub uri: &'static str,
    pub metadata: Option<Arc<dyn MetaDataScraper>>,
    pub search: Option<Arc<dyn SearchScraper>>,
    pub reader: Option<Arc<dyn ReaderScraper>>,
}
