use std::{
    collections::{BTreeMap, HashSet},
    mem,
};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

#[derive(Default)]
pub struct FilterDuplicatesProcessor;

fn filter(item: ScrapedData) -> ScrapedData {
    match item {
        ScrapedData::Arr(scraped_datas) => {
            let mut seen = HashSet::new();
            let mut result = Vec::new();
            for val in scraped_datas.into_iter().map(filter) {
                if seen.insert(val.clone()) {
                    result.push(val);
                }
            }
            ScrapedData::Arr(result)
        }
        ScrapedData::Map(hash_map) => {
            ScrapedData::Map(hash_map.into_iter().map(|(k, v)| (k, filter(v))).collect())
        }
        s => s,
    }
}

impl PostMetadataProcessor for FilterDuplicatesProcessor {
    fn name(&self) -> &str {
        "filter_duplicates"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next();
        let key1 = match key1 {
            Some(v) => match v == "/" {
                true => None,
                false => Some(v),
            },
            None => None,
        };
        match key1 {
            Some(v) => {
                if let Some(item1) = data.remove(v.trim()) {
                    data.insert(v.trim().to_owned(), filter(item1));
                }
            }
            None => {
                let mut other = BTreeMap::new();
                mem::swap(&mut other, data);
                let mut other = filter(ScrapedData::Map(other)).as_map().unwrap();
                mem::swap(&mut other, data);
            }
        }
    }
}
