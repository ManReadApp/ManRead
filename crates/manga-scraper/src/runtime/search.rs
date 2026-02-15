use api_structure::search::ValidSearches;
use async_trait::async_trait;
use scraper::Html;
use scraper_module::{
    ExternalSearchResponse, ScrapedSearchResponse, ScraperError, ScraperResult, SearchQuery,
    SearchScraper,
};

use crate::init::parse::InterpretedSearch;

#[async_trait]
impl SearchScraper for InterpretedSearch {
    fn query(&self) -> ValidSearches {
        ValidSearches::QueryOffset
    }

    async fn search(&self, query: SearchQuery) -> ScraperResult<ExternalSearchResponse> {
        let (page, selectors, query) = match query {
            SearchQuery::Simple(data) => {
                let query = data
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let page = data.get("page").and_then(|v| v.as_int()).unwrap_or(1);
                (
                    page,
                    match query.is_empty() {
                        true => &self.no_query,
                        false => &self.query,
                    },
                    query,
                )
            }
            _ => return Err(ScraperError::InvalidQuery),
        };
        let resp = self
            .engine
            .request_str(
                true,
                &selectors
                    .url
                    .replace("{query}", &query)
                    .replace("{page}", &page.to_string()),
            )
            .await
            .unwrap();
        let html = Html::parse_document(resp.as_str());
        let v = html.select(&selectors.selector).collect::<Vec<_>>();
        let label = selectors.label.as_ref().or(selectors.a.as_ref());
        let mut temp = v
            .into_iter()
            .map(|v| ScrapedSearchResponse {
                title: match &label {
                    Some(a) => v.select(&a).next().unwrap(),
                    None => v,
                }
                .text()
                .collect::<String>()
                .trim()
                .to_owned(),
                url: match &selectors.a {
                    Some(a) => v.select(&a).next().unwrap(),
                    None => v,
                }
                .attr("href")
                .unwrap()
                .to_owned(),
                cover: selectors.cover.as_ref().map(|sel| {
                    let item = v.select(&sel).next().unwrap();
                    item.attr("data-src")
                        .or_else(|| item.attr("src"))
                        .unwrap()
                        .to_owned()
                }),
                ty: selectors
                    .ty
                    .as_ref()
                    .map(|sel| v.select(&sel).next().unwrap().text().collect::<String>()),
                status: selectors
                    .status
                    .as_ref()
                    .and_then(|sel| v.select(&sel).next())
                    .map(|v| v.text().collect::<String>().trim().to_owned()),
            })
            .collect();
        for (item, query) in self.post_search.iter() {
            temp = item.process(&selectors.url, temp, query);
        }
        let (last_page, prev_page, next_page) = match selectors.single_page.unwrap_or_default() {
            true => (None, None, None),
            false => {
                let pages = html
                    .select(&selectors.pages.as_ref().unwrap())
                    .map(|v| match &selectors.pages_attr {
                        None => Some(v.text().collect::<String>().trim().to_owned()),
                        Some(attr) => v.attr(attr).map(|v| v.trim().to_owned()),
                    })
                    .collect::<Vec<_>>();

                let last_page: usize = pages
                    .into_iter()
                    .flatten()
                    .filter_map(|v| match &selectors.pages_regex {
                        None => Some(v),
                        Some(regex) => regex.captures(&v).map(|m| m[1].to_owned()),
                    })
                    .filter(|v| !["...", "Next »", "« Prev"].contains(&v.as_str()))
                    .map(|v| v.parse::<usize>().expect(&v))
                    .max()
                    .unwrap();

                match selectors.next_only.unwrap_or_default() {
                    true => (
                        None,
                        match page > 1 {
                            true => Some(page as usize - 1),
                            false => None,
                        },
                        Some(last_page),
                    ),
                    false => (
                        Some(last_page),
                        match page > 1 {
                            true => Some(page as usize - 1),
                            false => None,
                        },
                        match page < last_page as i64 {
                            true => Some(page as usize + 1),
                            false => None,
                        },
                    ),
                }
            }
        };

        Ok(ExternalSearchResponse {
            items: temp,
            next_page,
            prev_page,
            last_page,
            page: page as usize,
        })
    }
}
