use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

use super::flatten::add_to_map;

#[derive(Default)]
pub struct ArrayToMapPostProcessor;

impl PostMetadataProcessor for ArrayToMapPostProcessor {
    fn name(&self) -> &str {
        "array_to_map"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let key = target.split(" ").next().unwrap().trim();
        let item = data.remove(key);
        match item {
            Some(ScrapedData::Arr(arr)) => {
                if arr.len() % 2 == 0 {
                    let chunks = arr
                        .chunks_exact(2)
                        .map(|v| (v[0].get_str(), v[1].clone()))
                        .collect::<Vec<_>>();
                    let cancel = chunks.iter().any(|v| v.0.is_none());
                    if cancel {
                        data.insert(key.to_owned(), ScrapedData::Arr(arr));
                        return;
                    }
                    for (k, v) in chunks {
                        add_to_map(data, k.unwrap(), v);
                    }
                } else {
                    data.insert(key.to_owned(), ScrapedData::Arr(arr));
                }
            }
            Some(item) => {
                data.insert(key.to_owned(), item);
            }
            _ => {}
        }
    }
}
