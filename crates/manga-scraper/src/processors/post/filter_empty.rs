use scraper_module::ScrapedSearchResponse;

use crate::processors::PostSearchProcessor;

#[derive(Default)]
pub struct FilterEmptyPostProcessor;

impl PostSearchProcessor for FilterEmptyPostProcessor {
    fn name(&self) -> &str {
        "filter_empty"
    }

    fn process(
        &self,
        _: &str,
        data: Vec<ScrapedSearchResponse>,
        target: &str,
    ) -> Vec<ScrapedSearchResponse> {
        let target = target.trim();
        match target {
            "cover" => data
                .into_iter()
                .map(|mut v| {
                    v.cover = v.cover.and_then(|v| match v.is_empty() {
                        true => None,
                        false => Some(v),
                    });
                    v
                })
                .collect(),
            "status" => data
                .into_iter()
                .map(|mut v| {
                    v.status = v.status.and_then(|v| match v.is_empty() {
                        true => None,
                        false => Some(v),
                    });
                    v
                })
                .collect(),
            "ty" => data
                .into_iter()
                .map(|mut v| {
                    v.ty = v.ty.and_then(|v| match v.is_empty() {
                        true => None,
                        false => Some(v),
                    });
                    v
                })
                .collect(),
            _ => data,
        }
    }
}
