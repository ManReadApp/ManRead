use scraper_module::ScrapedSearchResponse;
use url::Url;

use crate::processors::PostSearchProcessor;
#[derive(Default)]
pub struct AddBasePostProcessor;

impl PostSearchProcessor for AddBasePostProcessor {
    fn name(&self) -> &str {
        "add_base"
    }

    fn process(
        &self,
        url: &str,
        data: Vec<ScrapedSearchResponse>,
        target: &str,
    ) -> Vec<ScrapedSearchResponse> {
        let target = target.trim();
        match target {
            "url" => data
                .into_iter()
                .map(|mut v| {
                    v.url = add_base(url, v.url);
                    v
                })
                .collect(),
            "cover" => data
                .into_iter()
                .map(|mut v| {
                    v.cover = v.cover.map(|v| add_base(url, v));
                    v
                })
                .collect(),
            _ => data,
        }
    }
}

fn add_base(base: &str, url: String) -> String {
    if url.starts_with("http") {
        url
    } else {
        Url::parse(base).unwrap().join(&url).unwrap().to_string()
    }
}
