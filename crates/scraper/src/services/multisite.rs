use crate::downloader::download;
use crate::pages::hidden;
use crate::services::icon::get_uri;
use crate::services::{config_to_request_builder, Service};
use crate::{ExternalSite, ScrapeError};
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::scrape::ScrapeAccount;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use url::Url;

#[derive(Default)]
pub struct MultiSiteService {
    client: Client,
    services: HashMap<String, Service>,
    internal: HashSet<&'static str>,
}

impl MultiSiteService {
    pub fn new(services: HashMap<String, Service>) -> Self {
        Self {
            internal: hidden::multi::register().into_iter().collect(),
            client: Default::default(),
            services,
        }
    }

    pub fn get_services(&self) -> Vec<String> {
        let mut uris: Vec<String> = self.services.keys().cloned().collect();
        uris.append(&mut self.internal.iter().map(|v| v.to_string()).collect());
        uris
    }

    pub async fn get_chapters(
        &self,
        url: &str,
        data: Arc<Vec<ExternalSite>>,
    ) -> Result<(Vec<Info>, Vec<Info>), ScrapeError> {
        let uri = get_uri(&data, url)?;
        let url = modify_url(&self.client, &uri, url).await;
        let (mut now, mut later) = {
            if let Some(v) = self.services.get(&uri) {
                let req = config_to_request_builder(&self.client, &v.config, &url);
                let html = download(req, v.cf_bypass()).await?;
                let mut fields = v.process(html.as_str());
                fields.insert("req_url".to_string(), url.clone());
                let items = post_process(&self.client, uri.as_str(), fields)
                    .await
                    .map(|v| {
                        v.into_iter()
                            .map(|mut v| {
                                if v.url.starts_with('/') || !v.url.starts_with("http") {
                                    let mut url_base = Url::parse(&url)?;
                                    let _ = url_base.set_scheme("https");
                                    v.url = format!(
                                        "{}/{}",
                                        url_base.origin().ascii_serialization(),
                                        v.url.strip_prefix("/").unwrap_or(&v.url)
                                    );
                                }
                                Ok(v)
                            })
                            .collect::<Result<Vec<_>, ScrapeError>>()
                    })
                    .map(|v| Ok((v?, vec![])))?;
                match items {
                    Ok(v) => Ok(v),
                    Err(e) => match manual(&self.client, uri.as_str(), &url).await {
                        Ok(v) => Ok(v),
                        Err(_) => Err(e),
                    },
                }
            } else {
                manual(&self.client, uri.as_str(), &url).await
            }
        }?;
        let mut existing = HashSet::new();
        let mut last = 0.0;
        let fix = false;
        for now in &mut now {
            let mut increase = true;
            if now.episode == 0.0 {
                if fix {
                    increase = true;
                    now.episode = last;
                } else {
                    return Err(ScrapeError::input_error("failed to parse episode"));
                }
            }
            while existing.contains(&format!("{:.2}", now.episode)) {
                if increase {
                    now.episode += 0.01;
                } else {
                    return Err(ScrapeError::input_error("episode does already exist"));
                }
            }
            now.episode = cut_float(now.episode);
            last = now.episode;

            existing.insert(format!("{:.2}", now.episode));
        }
        for later in &mut later {
            if later.episode == 0.0 {
                return Err(ScrapeError::input_error("failed to parse episode"));
            }
            if existing.contains(&format!("{:.2}", later.episode)) {
                return Err(ScrapeError::input_error("episode does already exist"));
            }
            existing.insert(format!("{:.2}", later.episode));
        }
        Ok((now, later))
    }

    pub async fn get_pages(
        &self,
        info: Info,
        acc: Option<ScrapeAccount>,
    ) -> Result<Vec<String>, ScrapeError> {
        if let Some(v) = self.services.get(&info.site) {
            let req = config_to_request_builder(&self.client, &v.config, &info.url);
            let html = download(req, v.cf_bypass()).await?;
            let fields = v.process(html.as_str());
            post_process_pages(&self.client, info.site.as_str(), fields).await
        } else {
            manual_pages(&self.client, info, acc).await
        }
    }
}

fn cut_float(f: f64) -> f64 {
    format!("{:.2}", f).parse().expect("cant fail")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub site: String,
    pub url: String,
    pub titles: Vec<String>,
    pub episode: f64,
    pub account: Option<i64>,
}

impl Info {
    pub fn add_title(mut self, title: &Option<String>) -> Self {
        if let Some(v) = title {
            self.titles.push(v.to_string())
        }
        self
    }
}

pub fn parse_episode(s: &str) -> Result<f64, ScrapeError> {
    let re = Regex::new(r"chapter\s+(\d+(\.\d+)?)").expect("static");
    let re2 = Regex::new(r"ch\.\s+(\d+(\.\d+)?)").expect("static");
    if let Some(captured) = re.captures(&s.to_lowercase()) {
        let number_str = &captured[1];
        Ok(number_str.parse()?)
    } else if let Some(captured) = re2.captures(&s.to_lowercase()) {
        let number_str = &captured[1];
        Ok(number_str.parse()?)
    } else if let Some(captured) = Regex::new(r"第(\d+(\.\d+)?)").expect("static").captures(s) {
        let number_str = &captured[1];
        Ok(number_str.parse()?)
    } else {
        Err(ApiErr {
            message: Some("couldnt find chapter number".to_string()),
            cause: None,
            err_type: ApiErrorType::InternalError,
        }
        .into())
    }
}

async fn post_process(
    client: &Client,
    uri: &str,
    fields: HashMap<String, String>,
) -> Result<Vec<Info>, ScrapeError> {
    let err = |len1, len2| {
        if len1 != len2 || len2 == 0 {
            Err(ApiErr {
                message: Some("Ivalid labels/urls".to_string()),
                cause: None,
                err_type: ApiErrorType::InternalError,
            })
        } else {
            Ok(())
        }
    };
    if let Some(urls) = fields.get("urls") {
        let mut urls: Vec<String> = serde_json::from_str(urls)?;
        if uri == "asura" {
            urls = urls.into_iter().map(|v| format!("/series/{v}")).collect();
        }
        let mut res = vec![];
        if let Some(labels) = fields.get("labels") {
            let labels: Vec<String> = serde_json::from_str(labels)?;
            err(labels.len(), urls.len())?;
            for (i, url) in urls.into_iter().enumerate() {
                let title = labels
                    .get(i)
                    .ok_or(ScrapeError::node_not_found())?
                    .to_string();
                let episode = parse_episode(title.as_str()).unwrap_or(0.0);
                res.push(Info {
                    site: uri.to_string(),
                    url,
                    titles: vec![title],
                    episode,
                    account: None,
                })
            }
            return Ok(res);
        } else if let Some(episodes) = fields.get("episodes") {
            let episodes: Vec<String> = serde_json::from_str(episodes)?;
            err(episodes.len(), urls.len())?;
            for (i, url) in urls.into_iter().enumerate() {
                let title = episodes
                    .get(i)
                    .ok_or(ScrapeError::node_not_found())?
                    .replace('-', ".")
                    .to_string();
                let episode = title.parse()?;

                res.push(Info {
                    site: uri.to_string(),
                    url,
                    titles: vec![title],
                    episode,
                    account: None,
                })
            }
            return Ok(res);
        }
    }
    hidden::multi::post_process_info(client, uri, fields).await
}

async fn manual(
    client: &Client,
    uri: &str,
    url: &str,
) -> Result<(Vec<Info>, Vec<Info>), ScrapeError> {
    hidden::multi::manual_info(client, uri, url).await
}

async fn manual_pages(
    client: &Client,
    info: Info,
    acc: Option<ScrapeAccount>,
) -> Result<Vec<String>, ScrapeError> {
    hidden::multi::manual_pages(client, info, acc).await
}

async fn post_process_pages(
    client: &Client,
    uri: &str,
    fields: HashMap<String, String>,
) -> Result<Vec<String>, ScrapeError> {
    if let Some(v) = fields.get("imgs_gen") {
        let map: HashMap<String, Value> = serde_json::from_str(v)?;
        let items = map
            .get("sources")
            .ok_or(ScrapeError::node_not_found())?
            .as_array()
            .ok_or(ScrapeError::node_not_found())?;
        let imgs = items
            .first()
            .ok_or(ScrapeError::node_not_found())?
            .get("images")
            .ok_or(ScrapeError::node_not_found())?
            .as_array()
            .ok_or(ScrapeError::node_not_found())?;
        let imgs = imgs
            .into_iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        if imgs.is_empty() {
            return Err(ScrapeError::node_not_found());
        }
        return Ok(imgs.into_iter().map(|v| v.to_string()).collect());
    }
    if let Some(v) = fields.get("imgs_back").cloned() {
        let back: Vec<String> = serde_json::from_str(&v)?;
        if let Some(v) = fields.get("imgs") {
            let value: Vec<String> = serde_json::from_str(v)?;
            let urls = value
                .into_iter()
                .enumerate()
                .map(|(index, v)| match v.is_empty() {
                    true => back
                        .get(index)
                        .ok_or(ScrapeError::node_not_found())
                        .map(|v| v.clone()),
                    false => Ok(v),
                })
                .map(|url| url.map(|url| url.replace(['\t', '\n'], "")))
                .collect::<Result<_, _>>()?;
            Ok(urls)
        } else {
            hidden::multi::post_process_pages(client, uri, fields).await
        }
    } else if let Some(v) = fields.get("imgs") {
        let urls: Vec<String> = serde_json::from_str(v)?;
        Ok(urls
            .into_iter()
            .map(|url| url.replace(['\t', '\n'], ""))
            .collect())
    } else {
        hidden::multi::post_process_pages(client, uri, fields).await
    }
}

pub async fn modify_url(client: &Client, uri: &str, url: &str) -> String {
    if let Some(v) = hidden::multi::modify_url(client, uri, url).await {
        v
    } else {
        url.to_string()
    }
}
