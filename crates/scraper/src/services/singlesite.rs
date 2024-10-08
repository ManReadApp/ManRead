use crate::downloader::download;
use crate::error::ScrapeError;
use crate::pages;
use crate::services::icon::{get_uri, ExternalSite};
use crate::services::{config_to_request_builder, Service};
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Default)]
pub struct SingleSiteService {
    client: Client,
    services: HashMap<String, Service>,
    internal: HashSet<&'static str>,
}

impl SingleSiteService {
    pub fn new(services: HashMap<String, Service>) -> Self {
        Self {
            client: Default::default(),
            services,
            internal: pages::hidden::single::register().into_iter().collect(),
        }
    }

    pub fn get_services(&self) -> Vec<String> {
        let mut uris: Vec<String> = self.services.keys().cloned().collect();
        uris.append(&mut self.internal.iter().map(|v| v.to_string()).collect());
        uris
    }

    pub async fn get_pages(
        &self,
        url: &str,
        data: Arc<Vec<ExternalSite>>,
    ) -> Result<Vec<String>, ScrapeError> {
        let uri = get_uri(&data, url)?;
        if let Some(v) = self.services.get(&uri) {
            let req = config_to_request_builder(&self.client, &v.config, url);
            let html = download(req, v.cf_bypass()).await?;
            let fields = v.process(html.as_str());
            post_process(&uri, fields)
        } else {
            manual(&uri, url).await
        }
    }
}

fn post_process(uri: &str, values: HashMap<String, String>) -> Result<Vec<String>, ScrapeError> {
    if let Some(v) = values.get("imgs") {
        return Ok(serde_json::from_str(v)?);
    }
    if let Some(v) = values.get("imgs_filter") {
        let items: Vec<String> = serde_json::from_str(v)?;
        return Ok(items.into_iter().filter(|v| !v.is_empty()).collect());
    }
    pages::hidden::single::post_process(uri, values)
}

async fn manual(uri: &str, url: &str) -> Result<Vec<String>, ScrapeError> {
    pages::hidden::single::manual(uri, url).await
}
