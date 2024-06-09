use crate::downloader::download;
use crate::services::config_to_request_builder;
use crate::ScrapeError;
use api_structure::scraper::ScrapeSearchResult;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

pub mod parser;

#[derive(Deserialize)]
pub struct SearchServiceDeserialized {
    headers: Option<String>,
    url_empty: Option<String>,
    url: String,
    selector: String,
    label_selector: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    status: Option<String>,
    cover: String,
    offset: Option<u32>,
}

impl SearchServiceDeserialized {
    pub(crate) fn convert(self, path: &Path) -> SearchServiceScrapeData {
        let headers = match self.headers {
            None => HashMap::new(),
            Some(header) => {
                serde_json::from_str(&read_to_string(path.join(header)).unwrap()).unwrap()
            }
        };

        SearchServiceScrapeData {
            headers,
            url_empty: self.url_empty,
            url: self.url,
            selector: Selector::parse(&self.selector).unwrap(),
            cover: Selector::parse(&self.cover).unwrap(),
            label_selector: self.label_selector.map(|v| Selector::parse(&v).unwrap()),
            type_: self.type_.map(|v| Selector::parse(&v).unwrap()),
            status: self.status.map(|v| Selector::parse(&v).unwrap()),
            offset: self.offset,
        }
    }
}

pub struct SearchServiceScrapeData {
    headers: HashMap<String, String>,
    url_empty: Option<String>,
    url: String,
    selector: Selector,
    cover: Selector,
    label_selector: Option<Selector>,
    type_: Option<Selector>,
    status: Option<Selector>,
    offset: Option<u32>,
}

impl SearchServiceScrapeData {
    pub async fn search(
        &self,
        client: &Client,
        query: String,
        page: u32,
    ) -> Result<Vec<ScrapeSearchResult>, ScrapeError> {
        let mut url = None;
        if query.is_empty() {
            if let Some(u) = &self.url_empty {
                url = Some(u.clone())
            }
        }
        if url.is_none() {
            url = Some(self.url.clone())
        }
        let url = url
            .unwrap()
            .replace("{query}", &urlencoding::encode(&query))
            .replace("{page}", &page.to_string())
            .replace(
                "{offset}",
                &((page - 1) * self.offset.unwrap_or(0)).to_string(),
            );
        let html = download(config_to_request_builder(client, &self.headers, &url)).await?;
        let doc = Html::parse_document(html.as_str());
        let urls = doc
            .select(&self.selector)
            .map(|v| v.attr("href").unwrap_or_default().to_string())
            .collect::<Vec<_>>();

        let cover = doc
            .select(&self.cover)
            .map(|v| {
                v.attr("src")
                    .unwrap_or(v.attr("data-src").unwrap_or_default())
                    .to_string()
            })
            .map(|v| {
                v.split_once("/https://")
                    .map(|(_, url)| format!("https://{url}"))
                    .unwrap_or(v)
            })
            .collect::<Vec<_>>();
        let labels: Vec<String> = if let Some(label) = &self.label_selector {
            doc.select(label)
                .map(|v| v.text().collect())
                .collect::<Vec<_>>()
        } else {
            doc.select(&self.selector)
                .map(|v| v.text().collect())
                .collect::<Vec<_>>()
        };

        let status: Option<Vec<String>> = self.status.as_ref().map(|status| {
            doc.select(status)
                .map(|v| v.text().collect())
                .collect::<Vec<_>>()
        });

        let type_: Option<Vec<String>> = self.type_.as_ref().map(|type_| {
            doc.select(type_)
                .map(|v| v.text().collect())
                .collect::<Vec<_>>()
        });
        let mut res = vec![];
        for (i, url) in urls.into_iter().enumerate() {
            res.push(ScrapeSearchResult {
                title: labels.get(i).unwrap().to_string(),
                url,
                cover: cover.get(i).unwrap().to_string(),
                r#type: type_.as_ref().map(|v| v.get(i).unwrap().to_string()),
                status: status.as_ref().map(|v| v.get(i).unwrap().to_string()),
            })
        }
        Ok(res)
    }
}
