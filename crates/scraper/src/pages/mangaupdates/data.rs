use crate::downloader::download;
use crate::services::metadata::ItemOrArray;
use crate::ScrapeError;

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn get_data(
    client: &Client,
    url: &str,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    let text = download(client.get(url), false).await?;
    let re = Regex::new(r#"https://api\.mangaupdates\.com/v1/series/([a-zA-Z0-9]+)/rss"#)
        .expect("static");
    if let Some(v) = re.captures(&text) {
        let url = format!("https://api.mangaupdates.com/v1/series/{}", &v[1]);
        let text = download(client.get(url), false).await?;
        let mut hm = HashMap::new();
        let json: InfoResponse = serde_json::from_str(&text)?;
        let mut titles = vec![json.title];
        titles.append(&mut json.associated.into_iter().map(|v| v.title).collect());
        hm.insert("titles", ItemOrArray::Array(titles));
        hm.insert("url", ItemOrArray::Item(json.url));
        hm.insert("img", ItemOrArray::Item(json.image.url.original));
        hm.insert("description", ItemOrArray::Item(json.description));
        hm.insert("type", ItemOrArray::Item(json.r#type));
        let mut tags: Vec<String> = json.genres.into_iter().map(|v| v.genre).collect();
        tags.append(&mut json.categories.into_iter().map(|c| c.category).collect());
        hm.insert("tags", ItemOrArray::Array(tags));
        hm.insert("year", ItemOrArray::Item(json.year));
        hm.insert("status", ItemOrArray::Item(json.status));
        hm.insert("licensed", ItemOrArray::Item(json.licensed.to_string()));
        hm.insert("completed", ItemOrArray::Item(json.completed.to_string()));
        hm.insert(
            "related",
            ItemOrArray::Array(
                json.related_series
                    .into_iter()
                    .map(|v| {
                        format!(
                            "{}: {} | {}",
                            v.relation_type,
                            v.related_series_name.unwrap_or("UNKNOWN".to_string()),
                            v.related_series_id
                        )
                    })
                    .collect(),
            ),
        );
        hm.insert(
            "authors",
            ItemOrArray::Array(
                json.authors
                    .into_iter()
                    .map(|v| format!("{}: {}", v.r#type, v.name))
                    .collect(),
            ),
        );
        hm.insert(
            "publishers",
            ItemOrArray::Array(
                json.publishers
                    .into_iter()
                    .map(|v| format!("{}: {}", v.r#type, v.publisher_name))
                    .collect(),
            ),
        );
        Ok(hm
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect())
    } else {
        Err(ScrapeError::node_not_found())
    }
}

#[derive(Serialize, Deserialize)]
struct Publisher {
    pub publisher_name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize)]
struct Author {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize)]
struct Genre {
    pub genre: String,
}

#[derive(Serialize, Deserialize)]
struct Url {
    pub original: String,
    pub thumb: String,
}

#[derive(Serialize, Deserialize)]
struct Image {
    pub url: Url,
    pub height: i64,
    pub width: i64,
}

#[derive(Serialize, Deserialize)]
struct OtherTitles {
    pub title: String,
}

#[derive(Serialize, Deserialize)]
struct InfoResponse {
    pub title: String,
    pub url: String,
    pub associated: Vec<OtherTitles>,
    pub description: String,
    pub image: Image,
    #[serde(rename = "type")]
    pub r#type: String,
    pub year: String,
    pub genres: Vec<Genre>,
    pub categories: Vec<Category>,
    pub status: String,
    pub licensed: bool,
    pub completed: bool,
    pub related_series: Vec<RelatedSeries>,
    pub authors: Vec<Author>,
    pub publishers: Vec<Publisher>,
}

#[derive(Serialize, Deserialize)]
struct RelatedSeries {
    pub relation_type: String,
    pub related_series_id: i64,
    pub related_series_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Category {
    pub category: String,
}
