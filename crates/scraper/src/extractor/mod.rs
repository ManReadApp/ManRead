use crate::downloader::download;
use crate::services::config_to_request_builder;
use crate::ScrapeError;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use url::Url;

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
    cover_data_src: Option<bool>,
    status: Option<String>,
    cover: String,
    offset: Option<u32>,
}

impl SearchServiceDeserialized {
    pub(crate) fn convert(self, path: &Path) -> Result<SearchServiceScrapeData, ScrapeError> {
        let headers = match self.headers {
            None => HashMap::new(),
            Some(header) => serde_json::from_str(
                &read_to_string(path.join(header)).map_err(|v| ScrapeError::init_file(path, v))?,
            )
            .map_err(|e| ScrapeError::init_file(path, e))?,
        };

        Ok(SearchServiceScrapeData {
            cf_bypass: headers.get("cf_bypass") == Some(&"true".to_string()),
            headers,
            url_empty: self.url_empty,
            url: self.url,
            selector: Selector::parse(&self.selector)
                .map_err(|e| ScrapeError::init_file(path, e))?,
            cover: Selector::parse(&self.cover).map_err(|e| ScrapeError::init_file(path, e))?,
            cover_data_src: self.cover_data_src.unwrap_or_default(),
            label_selector: self
                .label_selector
                .as_ref()
                .map(|v| Selector::parse(v))
                .transpose()
                .map_err(|e| ScrapeError::init_file(path, e))?,
            type_: self
                .type_
                .as_ref()
                .map(|v| Selector::parse(v))
                .transpose()
                .map_err(|e| ScrapeError::init_file(path, e))?,
            status: self
                .status
                .as_ref()
                .map(|v| Selector::parse(v))
                .transpose()
                .map_err(|e| ScrapeError::init_file(path, e))?,
            offset: self.offset,
        })
    }
}

pub struct SearchServiceScrapeData {
    headers: HashMap<String, String>,
    url_empty: Option<String>,
    url: String,
    selector: Selector,
    cover: Selector,
    cover_data_src: bool,
    label_selector: Option<Selector>,
    type_: Option<Selector>,
    status: Option<Selector>,
    offset: Option<u32>,
    cf_bypass: bool,
}

impl SearchServiceScrapeData {
    pub async fn search(
        &self,
        client: &Client,
        query: String,
        page: u32,
    ) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
        let mut url = None;
        if query.is_empty() {
            if let Some(u) = &self.url_empty {
                url = Some(u.clone())
            }
        }
        if url.is_none() {
            url = Some(self.url.clone())
        }
        let url = url.unwrap_or_default();
        if !url.contains("{page}") && !url.contains("{offset}") && page > 1 {
            return Ok(vec![]);
        }
        let contain_query = url.contains("{query}");
        let url = url
            .replace("{query}", &urlencoding::encode(&query))
            .replace("{page}", &page.to_string())
            .replace(
                "{offset}",
                &((page - 1) * self.offset.unwrap_or(0)).to_string(),
            );

        let html = download(
            config_to_request_builder(client, &self.headers, &url),
            self.cf_bypass,
        )
        .await?;
        let origin = Url::parse(&url)
            .map(|v| v.origin().ascii_serialization())
            .map_err(|e| ScrapeError::input_error_trace("no origin", e))?;
        let doc = Html::parse_document(html.as_str());
        let urls = doc
            .select(&self.selector)
            .map(|v| v.attr("href").unwrap_or_default().to_string())
            .map(|v| match v.starts_with("/") {
                true => format!("{origin}{v}"),
                false => v,
            })
            .collect::<Vec<_>>();

        let cover = doc
            .select(&self.cover)
            .map(|v| match self.cover_data_src {
                true => v.attr("data-src").unwrap_or_default().to_string(),
                false => v
                    .attr("src")
                    .unwrap_or(v.attr("data-src").unwrap_or_default())
                    .to_string(),
            })
            .map(|v| {
                v.split_once("/https://")
                    .map(|(_, url)| format!("https://{url}"))
                    .unwrap_or(v)
            })
            .collect::<Vec<_>>();
        let labels: Vec<String> = if let Some(label) = &self.label_selector {
            doc.select(label)
                .map(|v| v.text().collect::<String>().trim().to_string())
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
            res.push(ScrapeSearchResponse {
                title: labels
                    .get(i)
                    .ok_or(|v| ScrapeError::node_not_found())
                    .to_string(),
                url,
                cover: cover
                    .get(i)
                    .ok_or(|v| ScrapeError::node_not_found())
                    .to_string(),
                r#type: type_
                    .as_ref()
                    .map(|v| {
                        v.get(i)
                            .map(|v| v.to_string())
                            .ok_or(|v| ScrapeError::node_not_found())
                    })
                    .transpose()?,
                status: status
                    .as_ref()
                    .map(|v| {
                        v.get(i)
                            .map(|v| v.to_string())
                            .ok_or(|v| ScrapeError::node_not_found())
                    })
                    .transpose()?,
            })
        }
        if !query.is_empty() && !contain_query {
            res = res
                .into_iter()
                .filter(|v| v.title.to_lowercase().contains(&query.to_lowercase()))
                .collect()
        }
        Ok(res)
    }
}
