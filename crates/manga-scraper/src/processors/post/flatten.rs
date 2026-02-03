use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

#[derive(Default)]
pub struct FlattenPostProcessor;

impl PostMetadataProcessor for FlattenPostProcessor {
    fn name(&self) -> &str {
        "flatten"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let key = target.split(" ").next().unwrap().trim();
        let item = data.remove(key);
        match item {
            Some(ScrapedData::Map(map)) => {
                for (k, v) in map {
                    add_to_map(data, k, v);
                }
            }
            Some(ScrapedData::Arr(arr)) => {
                if arr.iter().all(|v| v.as_map().is_some()) {
                    for item in arr {
                        let map = item.as_map().unwrap();
                        for (k, v) in map {
                            add_to_map(data, k, v);
                        }
                    }
                } else {
                    data.insert(key.to_owned(), ScrapedData::Arr(arr));
                }
            }
            Some(item) => {
                data.insert(key.to_owned(), item);
            }
            None => {}
        }
    }
}

pub fn add_to_map(data: &mut BTreeMap<String, ScrapedData>, k: String, v: ScrapedData) {
    let in_map = data.get_mut(&k);
    match in_map {
        None => {
            data.insert(k, v);
        }
        Some(ScrapedData::Str(str)) => {
            let str = str.clone();
            data.insert(k, ScrapedData::Arr(vec![ScrapedData::Str(str), v]));
        }
        Some(ScrapedData::Arr(arr)) => arr.push(v),
        _ => {}
    }
}
