use crate::processors::PostSearchProcessor;

#[derive(Default)]
pub struct RemoveAttrPostProcessor;

impl PostSearchProcessor for RemoveAttrPostProcessor {
    fn name(&self) -> &str {
        "remove_attr"
    }

    fn process(
        &self,
        _: &str,
        data: Vec<scraper_module::ScrapedSearchResponse>,
        target: &str,
    ) -> Vec<scraper_module::ScrapedSearchResponse> {
        match target {
            "url" => data
                .into_iter()
                .map(|mut v| {
                    v.url = v
                        .url
                        .split_once("?")
                        .map(|v| v.0.to_owned())
                        .unwrap_or(v.url);
                    v
                })
                .collect(),
            "cover" => data
                .into_iter()
                .map(|mut v| {
                    v.cover = v
                        .cover
                        .map(|v| v.split_once("?").map(|v| v.0.to_owned()).unwrap_or(v));
                    v
                })
                .collect(),
            _ => data,
        }
    }
}
