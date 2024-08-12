use crate::downloader::download;
use crate::pages::animeplanet::tags::TAGS;
use crate::pages::hidden::pages::mangadex::UA_ERR;
use crate::ScrapeError;
use api_structure::{
    models::manga::external_search::{SimpleSearch, ValidSearch},
    resp::manga::external_search::ScrapeSearchResponse,
};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

mod tags;

pub fn get_valid() -> ValidSearch {
    ValidSearch {
        sorts: vec![
            "title".to_string(),
            "published".to_string(),
            "most_read".to_string(),
            "most_reading".to_string(),
            "created".to_string(),
            "popular".to_string(),
        ],
        tags: TAGS.into_iter().map(|v| v.0.to_string()).collect(),
        status: vec![
            "finished".to_string(),
            "releasing".to_string(),
            "upcoming".to_string(),
        ],
    }
}

//https://www.anime-planet.com/manga/all?sort=title&order=asc
fn get_order(s: &str) -> &str {
    match s {
        "title" => "title",
        "published" => "year",
        "most_read" => "status_1",
        "most_reading" => "status_2",
        "created" => "recent",
        "popular" => "average",
        _ => unreachable!(),
    }
}

fn get_status(s: &str) -> &str {
    match s {
        "releasing" => "is_ongoing=1",
        "upcoming" => "is_unaired=1",
        "finished" => "is_completed=1",
        _ => unreachable!(),
    }
}

pub async fn search(
    client: &Client,
    search_request: SimpleSearch,
) -> Result<Vec<ScrapeSearchResponse>, ScrapeError> {
    let valid: ValidSearch = get_valid();
    if !search_request.validate(&valid) {
        return Err(ScrapeError::input_error("couldnt match ValidSearch"));
    }
    let mut items = vec![format!("page={}", search_request.page)];
    if !search_request.search.is_empty() {
        items.push(format!(
            "name={}",
            urlencoding::encode(&search_request.search)
        ))
    }
    if let Some(v) = &search_request.sort {
        let desc = match search_request.desc {
            true => "desc",
            false => "asc",
        };
        format!("sort={}&order={}", get_order(v), desc);
    }
    if let Some(v) = &search_request.status {
        items.push(get_status(v).to_string())
    }

    if !search_request.tags.is_empty() {
        let mut tag_ids = vec![];
        let tags = TAGS.into_iter().collect::<HashMap<_, _>>();
        for tag in &search_request.tags {
            tag_ids.push(*tags.get(tag.as_str()).unwrap());
        }
        items.push(format!(
            "include_tags={}",
            tag_ids
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ))
    }
    let url = format!("https://www.anime-planet.com/manga/all?{}", items.join("&"));
    let html = download(client.get(url).header(USER_AGENT, UA_ERR), false).await?;
    let doc = Html::parse_document(html.as_str());
    let mangas = Selector::parse(".card").unwrap();
    let title = Selector::parse(".cardName").unwrap();
    let cover = Selector::parse("img").unwrap();
    let url = Selector::parse("a").unwrap();
    let mut res = vec![];
    for manga in doc.select(&mangas) {
        let url = manga
            .select(&url)
            .next()
            .ok_or(ScrapeError::node_not_found())?;
        let cover = manga
            .select(&cover)
            .next()
            .ok_or(ScrapeError::node_not_found())?;
        let title = manga
            .select(&title)
            .next()
            .ok_or(ScrapeError::node_not_found())?;
        res.push(ScrapeSearchResponse {
            title: title.text().collect::<Vec<_>>().join(""),
            url: format!(
                "https://www.anime-planet.com{}",
                url.attr("href").ok_or(ScrapeError::node_not_found())?
            ),
            cover: cover
                .attr("data-src")
                .ok_or(ScrapeError::node_not_found())?
                .to_string(),
            r#type: None,
            status: None,
        });
    }

    Ok(res)
}
