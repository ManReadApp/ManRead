use crate::downloader::download;
use crate::services::metadata::ItemOrArray;
use crate::ScrapeError;
use api_structure::scraper::SimpleSearch;
use api_structure::scraper::ValidSearch;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

const QUERY: &str = "
query ($id: Int) { # Define which variables will be used in the query (id)
  Media (id: $id, type: MANGA) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
    title {
      romaji
      english
      native
    }
    coverImage {
      extraLarge
    }
    bannerImage
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    description
    type
    format
    genres
    status(version:2)
    source(version:3)
    synonyms
    isAdult
    countryOfOrigin
    isLicensed
    tags {
      name
    }
    studios {
      edges {
        isMain node{
          name
        }
      }
    }
    relations {
      edges {
        relationType(version:2)node {
          title{
            userPreferred
          }
          type
          coverImage {
            extraLarge
          }
        }
      }
    }
  }
}
";

pub async fn get_data(
    client: &Client,
    url: &str,
) -> Result<HashMap<String, ItemOrArray>, ScrapeError> {
    let mut id = url
        .split_once("anilist.co/manga/")
        .ok_or(ScrapeError::input_error("Invalid url"))?
        .1;
    if id.contains("/") {
        id = id.split_once("/").unwrap().0;
    }
    let json = json!({"query": QUERY, "variables": {"id": id}});
    let resp = download(
        client
            .post("https://graphql.anilist.co/")
            .header("Accept", "application/json")
            .json(&json),
    )
    .await?;
    let parsed: InfoResponse = serde_json::from_str(&resp)?;
    let mut media = parsed.data.media;
    let mut hm = HashMap::new();
    hm.insert("cover", ItemOrArray::Item(media.cover_image.extra_large));
    if let Some(banner_image) = media.banner_image {
        hm.insert("banner", ItemOrArray::Item(banner_image));
    }
    let mut titles = vec![
        format!("Native: {}", media.title.native),
        format!("Rative: {}", media.title.romaji),
        format!("English: {}", media.title.english),
    ];
    for synonym in media.synonyms {
        titles.push(format!("Unknown: {}", synonym));
    }
    hm.insert("titles", ItemOrArray::Array(titles));
    let mut tags: Vec<String> = media.tags.into_iter().map(|v| v.name).collect();
    tags.append(&mut media.genres);
    hm.insert("tags", ItemOrArray::Array(tags));
    hm.insert("licensed", ItemOrArray::Item(media.is_licensed.to_string()));
    hm.insert("adult", ItemOrArray::Item(media.is_adult.to_string()));
    hm.insert(
        "country_of_origin",
        ItemOrArray::Item(media.country_of_origin),
    );
    hm.insert("description", ItemOrArray::Item(media.description));
    if let Some(start_date) = media.start_date.display() {
        hm.insert("start_date", ItemOrArray::Item(start_date));
    }
    if let Some(end_date) = media.end_date.display() {
        hm.insert("end_date", ItemOrArray::Item(end_date));
    }
    hm.insert("format", ItemOrArray::Item(media.format));
    hm.insert("status", ItemOrArray::Item(media.status));
    hm.insert("source", ItemOrArray::Item(media.source));
    hm.insert("type", ItemOrArray::Item(media.r#type));
    let mut relations = vec![];
    for relation in media.relations.edges {
        relations.push(format!(
            "{} {}: {}",
            relation.relation_type, relation.node.r#type, relation.node.title.user_preferred,
        ));
    }
    hm.insert("relations", ItemOrArray::Array(relations));
    Ok(hm.into_iter().map(|v| (v.0.to_string(), v.1)).collect())
}

#[derive(Serialize, Deserialize)]
struct Title1 {
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Serialize, Deserialize)]
struct Node {
    pub title: Title1,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

#[derive(Serialize, Deserialize)]
struct Relation {
    #[serde(rename = "relationType")]
    pub relation_type: String,
    pub node: Node,
}

#[derive(Serialize, Deserialize)]
struct Relations {
    pub edges: Vec<Relation>,
}

#[derive(Serialize, Deserialize)]
struct Studios {
    pub edges: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
struct Tag {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct Media {
    pub title: Title,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    pub end_date: Date,
    pub description: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub format: String,
    pub genres: Vec<String>,
    pub status: String,
    pub source: String,
    pub synonyms: Vec<String>,
    #[serde(rename = "isAdult")]
    pub is_adult: bool,
    #[serde(rename = "countryOfOrigin")]
    pub country_of_origin: String,
    #[serde(rename = "isLicensed")]
    pub is_licensed: bool,
    pub tags: Vec<Tag>,
    pub studios: Studios,
    pub relations: Relations,
}

#[derive(Serialize, Deserialize)]
struct InfoResponse {
    pub data: Data,
}
#[derive(Serialize, Deserialize)]
struct Data {
    #[serde(rename = "Media")]
    pub media: Media,
}

#[derive(Serialize, Deserialize)]
struct Title {
    pub romaji: String,
    pub english: String,
    pub native: String,
}

#[derive(Serialize, Deserialize)]
struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: String,
}

#[derive(Serialize, Deserialize)]
struct Date {
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub day: Option<i64>,
}

impl Date {
    fn display(&self) -> Option<String> {
        if let Some(year) = self.year {
            if let Some(month) = self.month {
                if let Some(day) = self.day {
                    return Some(format!("{}-{}-{}", year, month, day));
                }
            }
        }
        None
    }
}

const QUERY2: &str = "query (
  $page: Int = 1
  $id: Int
  $type: MediaType
  $isAdult: Boolean = false
  $search: String
  $format: [MediaFormat]
  $status: MediaStatus
  $countryOfOrigin: CountryCode
  $source: MediaSource
  $season: MediaSeason
  $seasonYear: Int
  $year: String
  $onList: Boolean
  $yearLesser: FuzzyDateInt
  $yearGreater: FuzzyDateInt
  $episodeLesser: Int
  $episodeGreater: Int
  $durationLesser: Int
  $durationGreater: Int
  $chapterLesser: Int
  $chapterGreater: Int
  $volumeLesser: Int
  $volumeGreater: Int
  $licensedBy: [Int]
  $isLicensed: Boolean
  $genres: [String]
  $excludedGenres: [String]
  $tags: [String]
  $excludedTags: [String]
  $minimumTagRank: Int
  $sort: [MediaSort] = [POPULARITY_DESC, SCORE_DESC]
) {
  Page(page: $page, perPage: 50) {
    pageInfo {
      total
      perPage
      currentPage
      lastPage
      hasNextPage
    }
    media(
      id: $id
      type: $type
      season: $season
      format_in: $format
      status: $status
      countryOfOrigin: $countryOfOrigin
      source: $source
      search: $search
      onList: $onList
      seasonYear: $seasonYear
      startDate_like: $year
      startDate_lesser: $yearLesser
      startDate_greater: $yearGreater
      episodes_lesser: $episodeLesser
      episodes_greater: $episodeGreater
      duration_lesser: $durationLesser
      duration_greater: $durationGreater
      chapters_lesser: $chapterLesser
      chapters_greater: $chapterGreater
      volumes_lesser: $volumeLesser
      volumes_greater: $volumeGreater
      licensedById_in: $licensedBy
      isLicensed: $isLicensed
      genre_in: $genres
      genre_not_in: $excludedGenres
      tag_in: $tags
      tag_not_in: $excludedTags
      minimumTagRank: $minimumTagRank
      sort: $sort
      isAdult: $isAdult
    ) {
      id
      title {
        userPreferred
      }
      coverImage {
        extraLarge
        large
        color
      }
      startDate {
        year
        month
        day
      }
      endDate {
        year
        month
        day
      }
      bannerImage
      season
      seasonYear
      description
      type
      format
      status(version: 2)
      episodes
      duration
      chapters
      volumes
      genres
      isAdult
      averageScore
      popularity
      nextAiringEpisode {
        airingAt
        timeUntilAiring
        episode
      }
      mediaListEntry {
        id
        status
      }
      studios(isMain: true) {
        edges {
          isMain
          node {
            id
            name
          }
        }
      }
    }
  }
}
";

fn get_sort(s: &str, desc: bool) -> Value {
    let desc = match desc {
        true => "DESC",
        false => "ASC",
    };
    match s {
        "popularity" => serde_json::to_value(format!("POPULARITY_{}", desc)).unwrap(),
        "score" => serde_json::to_value(format!("SCORE_{}", desc)).unwrap(),
        "trending" => {
            serde_json::to_value([format!("TRENDING_{}", desc), format!("POPULARITY_{}", desc)])
                .unwrap()
        }
        "created" => serde_json::to_value(format!("ID_{}", desc)).unwrap(),
        "updated" => serde_json::to_value(format!("START_DATE_{}", desc)).unwrap(),
        _ => unreachable!(),
    }
}

fn get_status(s: &str) -> &str {
    match s {
        "releasing" => "RELEASING",
        "finished" => "FINISHED",
        "upcoming" => "NOT_YET_RELEASED",
        "hiatus" => "HIATUS",
        "cancelled" => "CANCELLED",
        _ => unreachable!(),
    }
}
pub async fn search(
    client: &Client,
    search: &SimpleSearch,
    tags: Vec<String>,
) -> Result<(), ScrapeError> {
    let valid: ValidSearch = ValidSearch::anilist();
    if !search.validate(&valid) {
        return Err(ScrapeError::input_error("couldnt match ValidSearch"));
    }
    let mut items = vec![
        ("page", serde_json::to_value(search.page).unwrap()),
        ("type", serde_json::to_value("MANGA").unwrap()),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();
    if let Some(query) = &search.search {
        if search.sort.is_none() {
            items.insert("sort", serde_json::to_value("SEARCH_MATCH").unwrap());
        }
        items.insert("search", serde_json::to_value(query).unwrap());
    } else {
        if search.sort.is_none() {
            items.insert(
                "sort",
                serde_json::to_value(["TRENDING_DESC", "POPULARITY_DESC"]).unwrap(),
            );
        }
    };
    if let Some(sort) = &search.sort {
        items.insert("sort", get_sort(sort, search.desc));
    }
    if let Some(status) = &search.status {
        items.insert("status", serde_json::to_value(get_status(status)).unwrap());
    }
    if !search.tags.is_empty() {
        items.insert("tags", serde_json::to_value(&search.tags).unwrap());
    }
    let json = json!({"query": QUERY2, "variables": items });

    let resp = download(
        client
            .post("https://graphql.anilist.co/")
            .header("Accept", "application/json")
            .json(&json),
    )
    .await?;
    println!("{}", resp);
    Ok(())
}

#[tokio::test]
async fn test() {
    //search(&Client::new(), &None).await.unwrap();
}
