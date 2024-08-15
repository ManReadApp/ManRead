use crate::downloader::download;
use crate::error::ScrapeError;
use crate::extractor::parser::clean_text;
use crate::pages;
use crate::pages::asuratoon::get_first_url;
use crate::pages::{anilist, kitsu};
use crate::services::icon::{get_uri, ExternalSite};
use crate::services::{config_to_request_builder, MangaData, Service};
use api_structure::error::{ApiErr, ApiErrorType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use url::Url;

#[derive(Default)]
pub struct MetaDataService {
    client: Client,
    services: HashMap<String, Service>,
    local_services: Arc<HashMap<String, HashMap<String, MangaData>>>,
}

impl MetaDataService {
    pub fn new(
        services: HashMap<String, Service>,
        local_services: Arc<HashMap<String, HashMap<String, MangaData>>>,
    ) -> Self {
        Self {
            client: Default::default(),
            services,
            local_services,
        }
    }

    pub async fn get_metadata(
        &self,
        url: &str,
        data: Arc<Vec<ExternalSite>>,
    ) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
        let uri = get_uri(&data, url)?;
        let mut url = self.process_url(&uri, url.to_string()).await;
        if let Some(v) = self.services.get(&uri) {
            let req = config_to_request_builder(&self.client, &v.config, url.as_str());
            let html = download(req, v.cf_bypass()).await?;
            let fields = v.process(html.as_str());
            post_process(&url, fields)
        } else if let Some(v) = self.local_services.get(&uri) {
            if !url.ends_with("/") {
                url = format!("{url}/");
            }
            v.get(&url)
                .ok_or(ScrapeError::invalid_url("url not found"))
                .map(|v| {
                    let mut data = HashMap::new();
                    data.insert("title".to_string(), ItemOrArray::Item(v.title.clone()));
                    data.insert("url".to_string(), ItemOrArray::Item(v.url.clone()));
                    data.insert(
                        "cover".to_string(),
                        ItemOrArray::Item(v.cover.clone().unwrap_or_default()),
                    );
                    for (key, value) in v.data.iter() {
                        data.insert(
                            key.clone(),
                            match value {
                                StringOrArr::String(v) => ItemOrArray::Item(v.clone()),
                                StringOrArr::Arr(v) => ItemOrArray::Array(v.clone()),
                            },
                        );
                    }
                    data
                })
        } else {
            manual(&self.client, &uri, &url).await
        }
    }
    async fn process_url(&self, uri: &str, url: String) -> String {
        if uri == "asura" {
            let html = download(self.client.get(&url), false).await.unwrap();
            get_first_url(
                &Url::parse(&url).unwrap().origin().ascii_serialization(),
                &html,
            )
            .unwrap()
            .to_string()
        } else {
            url
        }
    }
}

#[derive(Debug)]
pub enum ItemOrArray {
    Item(String),
    Array(Vec<String>),
    Map(HashMap<String, String>),
    ArrayDyn(Vec<Value>),
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrArr {
    String(String),
    Arr(Vec<String>),
}
fn post_process(
    url: &str,
    values: HashMap<String, String>,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    let mut res = HashMap::new();
    for (key, value) in values {
        let v;
        if let Ok(value) = serde_json::from_str(&value) {
            let value: Vec<Value> = value;
            let str = value.iter().all(|v| v.is_string());
            if str {
                v = ItemOrArray::Array(
                    value
                        .into_iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect(),
                )
            } else {
                v = ItemOrArray::ArrayDyn(value);
            }
        } else {
            v = ItemOrArray::Item(value);
        }
        res.insert(key, v);
    }
    let v = res.remove("rows");
    match v {
        Some(ItemOrArray::ArrayDyn(v)) => {
            for v in v {
                let v: (String, String) = serde_json::from_value(v).unwrap();
                let value: StringOrArr =
                    serde_json::from_str(&v.1).unwrap_or(StringOrArr::String(v.1));
                res.insert(
                    v.0,
                    match value {
                        StringOrArr::String(v) => ItemOrArray::Item(v),
                        StringOrArr::Arr(v) => ItemOrArray::Array(v),
                    },
                );
            }
        }
        Some(ItemOrArray::Array(v)) => {
            res.insert("rows".to_string(), ItemOrArray::Array(v));
        }
        Some(ItemOrArray::Item(v)) => {
            res.insert("rows".to_string(), ItemOrArray::Item(v));
        }
        None => {}
        Some(ItemOrArray::Map(v)) => {
            res.insert("rows".to_string(), ItemOrArray::Map(v));
        }
    }

    if let Some(ItemOrArray::Array(v)) = res.get("titles") {
        let mut titles = v.iter().map(|v| v.to_string()).collect::<HashSet<_>>();
        titles.remove("/");
        res.insert(
            "titles".to_string(),
            ItemOrArray::Array(titles.into_iter().collect()),
        );
    }
    if let Some(ItemOrArray::Array(v)) = res.get("tags") {
        let mut tags = v.iter().map(|v| v.to_string()).collect::<HashSet<_>>();
        tags.remove(",");
        res.insert(
            "tags".to_string(),
            ItemOrArray::Array(tags.into_iter().collect()),
        );
    }
    let v = res.remove("rows2");
    if let Some(ItemOrArray::Item(v)) = res.get("cover") {
        let origin = Url::parse(url).unwrap().origin().ascii_serialization();
        if v.starts_with("/") || !v.starts_with("http") {
            res.insert(
                "cover".to_string(),
                ItemOrArray::Item(format!("{origin}/{}", v.strip_prefix("/").unwrap_or(v))),
            );
        }
    }
    if let Some(ItemOrArray::Item(v)) = res.get("cover") {
        let cont = |input: &str| match input.rfind("?") {
            None => false,
            Some(v) => input[v..].contains("q=75"),
        };
        if cont(v) {
            res.insert(
                "cover".to_string(),
                ItemOrArray::Item(v.replace("q=75", "q=100")),
            );
        }
    }
    match v {
        None => {}
        Some(ItemOrArray::Array(v)) => {
            if v.len() % 2 != 0 {
                return Err(ScrapeError::input_error(
                    "not same ammount of keys & values",
                ));
            }

            let mut key = true;
            let mut key_v = "".to_string();
            for item in v {
                match key {
                    true => {
                        key_v = item;
                    }
                    false => {
                        res.insert(key_v.clone(), ItemOrArray::Item(item));
                    }
                }
                key = !key;
            }
        }
        Some(_) => {}
    }

    let v = res.remove("fields_labels");
    match v {
        Some(ItemOrArray::Array(v)) => {
            if let Some(ItemOrArray::Array(vv)) = res.remove("labels") {
                if v.len() == vv.len() {
                    for (i, data) in v.into_iter().enumerate() {
                        let value = vv.get(i).unwrap().as_str();
                        let text = clean_text(
                            clean_text(data)
                                .strip_prefix(value)
                                .ok_or(ScrapeError::node_not_found())?
                                .to_string(),
                        );
                        let key = value.replace(':', "");
                        match value {
                            "Genres:" | "Demographic:" | "Themes:" => {
                                let genres: Vec<String> = text
                                    .split(',')
                                    .map(|v| {
                                        v.split_once('\n').map(|v| v.0).unwrap_or(v).to_string()
                                    })
                                    .map(clean_text)
                                    .collect();
                                res.insert(key, ItemOrArray::Array(genres));
                            }
                            "Score:" | "Chapters:" | "Favorites:" | "Members:" | "Popularity:"
                            | "Volumes:" | "Ranked:" => {}
                            _ => {
                                res.insert(key, ItemOrArray::Item(text));
                            }
                        }
                    }
                }
            }
        }
        Some(ItemOrArray::ArrayDyn(v)) => {
            res.insert("fields_labels".to_string(), ItemOrArray::ArrayDyn(v));
        }
        Some(ItemOrArray::Map(v)) => {
            res.insert("fields_labels".to_string(), ItemOrArray::Map(v));
        }
        Some(ItemOrArray::Item(v)) => {
            res.insert("fields_labels".to_string(), ItemOrArray::Item(v));
        }
        None => {}
    }
    Ok(res)
}

async fn manual(
    client: &Client,
    uri: &str,
    url: &str,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    match uri {
        "manga-updates" => pages::mangaupdates::data::get_data(&client, url).await,
        "mangadex" => pages::mangadex::get_data(&client, url).await,
        "kitsu" => kitsu::get_data(client, url).await,
        "anilist" => anilist::get_data(client, url).await,
        _ => Err(ApiErr {
            message: Some("uri not registered".to_string()),
            cause: None,
            err_type: ApiErrorType::InternalError,
        }
        .into()),
    }
}
