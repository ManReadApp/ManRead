use crate::downloader::download;
use crate::error::ScrapeError;
use crate::extractor::parser::clean_text;
use crate::pages::asuratoon::get_first_url;
use crate::pages::{anilist, kitsu};
use crate::services::icon::{get_uri, ExternalSite};
use crate::services::{config_to_request_builder, Service};
use api_structure::error::{ApiErr, ApiErrorType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct MetaDataService {
    client: Client,
    services: HashMap<String, Service>,
}

impl MetaDataService {
    pub fn new(services: HashMap<String, Service>) -> Self {
        Self {
            client: Default::default(),
            services,
        }
    }

    pub async fn get_metadata(
        &self,
        url: &str,
        data: Arc<Vec<ExternalSite>>,
    ) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
        let uri = get_uri(&data, url)?;
        let url = self.process_url(&uri, url.to_string()).await;
        if let Some(v) = self.services.get(&uri) {
            let req = config_to_request_builder(&self.client, &v.config, url.as_str());
            let html = download(req).await?;
            let fields = v.process(html.as_str());
            post_process(fields)
        } else {
            manual(&self.client, &uri, &url).await
        }
    }
    async fn process_url(&self, uri: &str, url: String) -> String {
        if uri == "asura" {
            let html = download(self.client.get(url)).await.unwrap();
            get_first_url(&html).unwrap().to_string()
        } else {
            url
        }
    }
}

#[derive(Debug)]
pub enum ItemOrArray {
    Item(String),
    Array(Vec<String>),
    ArrayDyn(Vec<Value>),
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum StringOrArr {
    String(String),
    Arr(Vec<String>),
}
fn post_process(
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
        "manga-updates" => todo!(),
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
