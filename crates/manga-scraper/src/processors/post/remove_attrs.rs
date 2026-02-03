use crate::processors::{PostScraperProcessor, PostSearchProcessor};

#[derive(Default)]
pub struct RemoveAttrs;

impl PostScraperProcessor for RemoveAttrs {
    fn name(&self) -> &str {
        "remove_attrs"
    }

    fn process(&self, data: Vec<String>, _: &str) -> Vec<String> {
        data.into_iter()
            .map(|v| v.split_once("?").map(|v| v.0.to_owned()).unwrap_or(v))
            .collect()
    }
}

impl PostSearchProcessor for RemoveAttrs {
    fn name(&self) -> &str {
        "remove_attrs"
    }

    fn process(
        &self,
        _: &str,
        data: Vec<scraper_module::ScrapedSearchResponse>,
        _: &str,
    ) -> Vec<scraper_module::ScrapedSearchResponse> {
        data.into_iter()
            .map(|mut v| {
                if let Some(cover) = v.cover.clone() {
                    v.cover = Some(
                        cover
                            .split_once("?")
                            .map(|v| v.0.to_owned())
                            .unwrap_or(cover),
                    )
                }
                v
            })
            .collect()
    }
}
