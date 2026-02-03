use std::collections::BTreeMap;

use scraper_module::ScrapedData;

use crate::processors::{PostMetadataProcessor, PostScraperProcessor};

#[derive(Default)]
pub struct SplitProcessor;

fn split(v: &ScrapedData, key2: &str) -> ScrapedData {
    match v {
        ScrapedData::Str(s) => ScrapedData::Arr(
            s.split(key2)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
                .map(ScrapedData::Str)
                .collect::<Vec<_>>(),
        ),
        ScrapedData::Arr(scraped_datas) => {
            ScrapedData::Arr(scraped_datas.iter().map(|v| split(v, key2)).collect())
        }
        ScrapedData::Map(v) => ScrapedData::Map(
            v.into_iter()
                .map(|v| (v.0.clone(), split(v.1, key2)))
                .collect(),
        ),
        ScrapedData::Map2(items) => ScrapedData::Map2(
            items
                .into_iter()
                .map(|v| (v.0.clone(), split(&v.1, key2)))
                .collect(),
        ),
    }
}

impl PostScraperProcessor for SplitProcessor {
    fn name(&self) -> &str {
        "split"
    }

    fn process(&self, data: Vec<String>, target: &str) -> Vec<String> {
        data.into_iter()
            .flat_map(|v| {
                v.split(target.trim())
                    .map(|v| v.to_owned())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

impl PostMetadataProcessor for SplitProcessor {
    fn name(&self) -> &str {
        "split"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.splitn(2, " ");
        let key1 = keys.next().unwrap().trim();
        let mut key2 = keys.next().unwrap().trim();
        if key2 == "\\n" {
            key2 = "\n";
        }
        if let Some(v) = data.remove(key1) {
            let new = split(&v, key2);
            data.insert(key1.to_owned(), new.flatten_vec());
        }
    }
}
