use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

#[derive(Default)]
pub struct JoinProcessor;

impl PostMetadataProcessor for JoinProcessor {
    fn name(&self) -> &str {
        "join"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next().unwrap().trim();
        let key2 = keys.next().unwrap().trim();
        if let Some(item1) = data.remove(key1) {
            match item1 {
                ScrapedData::Arr(scraped_datas) => {
                    let items = scraped_datas
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>();
                    data.insert(
                        key1.to_owned(),
                        match items.len() == scraped_datas.len() {
                            true => ScrapedData::Str(items.join(key2)),
                            false => ScrapedData::Arr(scraped_datas),
                        },
                    )
                }
                v => data.insert(key1.to_owned(), v),
            };
        }
    }
}
