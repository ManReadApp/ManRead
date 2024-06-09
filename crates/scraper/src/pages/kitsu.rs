use crate::downloader::download;
use crate::services::metadata::ItemOrArray;
use crate::ScrapeError;
use api_structure::scraper::{ScrapeSearchResult, SimpleSearch};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub async fn get_data(
    client: &Client,
    url: &str,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    let mut slug = url
        .split_once("kitsu.io/manga/")
        .map(|v| v.1)
        .ok_or(ScrapeError::input_error("not a valid url"))?;
    if slug.contains('/') {
        slug = slug.split_once('/').unwrap().0;
    }
    let url = format!("https://kitsu.io/api/edge/manga?fields%5Bcategories%5D=slug%2Ctitle&filter%5Bslug%5D={slug}&include=categories,genres");
    let text = download(client.get(url)).await?;
    let mut parsed: MangaResponse = serde_json::from_str(&text)?;
    let data = parsed.data.remove(0);
    let mut hm = HashMap::new();
    let mut titles = vec![format!(
        "canonical_title: {}",
        data.attributes.canonical_title
    )];
    for (typ, title) in data.attributes.titles {
        titles.push(format!("{}: {}", typ, title));
    }

    for title in data.attributes.abbreviated_titles {
        titles.push(format!("unknown: {}", title))
    }
    hm.insert(
        "description",
        ItemOrArray::Item(data.attributes.description),
    );
    hm.insert("type", ItemOrArray::Item(data.r#type));
    hm.insert("subtype", ItemOrArray::Item(data.attributes.subtype));
    hm.insert(
        "cover",
        ItemOrArray::Item(data.attributes.cover_image.original),
    );
    hm.insert(
        "poster",
        ItemOrArray::Item(data.attributes.poster_image.original),
    );
    hm.insert("age_rating", ItemOrArray::Item(data.attributes.age_rating));
    hm.insert("status", ItemOrArray::Item(data.attributes.status));
    hm.insert("start_date", ItemOrArray::Item(data.attributes.start_date));
    hm.insert(
        "serialization",
        ItemOrArray::Item(data.attributes.serialization),
    );
    hm.insert(
        "tags",
        ItemOrArray::Array(
            parsed
                .included
                .into_iter()
                .map(|v| v.attributes.get_name())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
        ),
    );
    hm.insert("titles", ItemOrArray::Array(titles));
    Ok(hm.into_iter().map(|v| (v.0.to_string(), v.1)).collect())
}

#[derive(Serialize, Deserialize)]
struct MangaResponse {
    pub data: Vec<Info>,
    pub included: Vec<Data>,
}

#[derive(Serialize, Deserialize)]
struct Info {
    #[serde(rename = "type")]
    pub r#type: String,
    pub attributes: MangaAttributes,
}

#[derive(Serialize, Deserialize)]
struct MangaAttributes {
    pub description: String,
    pub titles: HashMap<String, String>,
    #[serde(rename = "canonicalTitle")]
    pub canonical_title: String,
    #[serde(rename = "abbreviatedTitles")]
    pub abbreviated_titles: Vec<String>,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "ageRating")]
    pub age_rating: String,
    pub subtype: String,
    pub status: String,
    #[serde(rename = "posterImage")]
    pub poster_image: Image,
    #[serde(rename = "coverImage")]
    pub cover_image: Image,
    pub serialization: String,
}

#[derive(Serialize, Deserialize)]
struct Image {
    pub original: String,
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub attributes: Attributes1,
}

#[derive(Serialize, Deserialize)]
struct Attributes1 {
    name: Option<String>,
    slug: String,
    title: Option<String>,
}

impl Attributes1 {
    fn get_name(self) -> String {
        if let Some(v) = self.name {
            v
        } else if let Some(v) = self.title {
            v
        } else {
            self.slug
        }
    }
}

pub async fn search(
    client: &Client,
    search: SimpleSearch,
) -> Result<Vec<ScrapeSearchResult>, ScrapeError> {
    let limit = 20;
    let offset = (search.page - 1) * limit;
    let mut url = format!("https://kitsu.io/api/edge/manga?fields%5Bmanga%5D=slug%2CcanonicalTitle%2Ctitles%2CposterImage%2Cdescription%2CaverageRating%2CstartDate%2CpopularityRank%2CratingRank&page%5Blimit%5D={limit}&page%5Boffset%5D={offset}");
    if !search.search.is_empty() {
        let query = urlencoding::encode(&search.search);
        url = format!("{url}&filter%5Btext%5D={query}")
    }
    if !search.tags.is_empty() {
        let categories = search.tags.join(",");
        url = format!("{url}&filter%5Bcategories%5D={categories}")
    }
    if let Some(sort) = search.sort {
        url = format!("{url}&sort={}", get_sort(&sort))
    }
    let text = download(client.get(url)).await?;
    let data: SearchResponse = serde_json::from_str(&text)?;
    let data = data.data;
    Ok(data
        .into_iter()
        .map(|v| ScrapeSearchResult {
            title: v.attributes.canonical_title,
            url: format!("https://kitsu.io/manga/{}", v.attributes.slug),
            cover: v.attributes.poster_image.original,
            r#type: Some(v.r#type),
            status: None,
        })
        .collect())
}

fn get_sort(s: &str) -> &str {
    match s {
        "popularity" => "-user_count",
        "rating" => "-average_rating",
        "updated" => "-start_date",
        "created" => "-created_at",
        _ => unreachable!(),
    }
}

#[derive(Serialize, Deserialize)]
struct PosterImage {
    pub original: String,
}

#[derive(Serialize, Deserialize)]
struct Titles {
    pub en: Option<String>,
    pub en_jp: Option<String>,
    pub en_us: Option<String>,
    pub ja_jp: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Attributes {
    pub slug: String,
    #[serde(rename = "canonicalTitle")]
    pub canonical_title: String,
    pub titles: Titles,
    #[serde(rename = "posterImage")]
    pub poster_image: PosterImage,
    pub description: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Struct {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Serialize, Deserialize)]
struct SearchResponse {
    pub data: Vec<Struct>,
}
