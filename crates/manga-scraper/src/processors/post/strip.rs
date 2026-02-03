use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;
#[derive(Default)]
pub struct StripSuffixPostProcessor;

impl PostMetadataProcessor for StripSuffixPostProcessor {
    fn name(&self) -> &str {
        "strip_suffix"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next().unwrap().trim();
        let key2 = keys.next().unwrap().trim();
        let item1 = data.remove(key1);
        let item1_str = item1.clone().and_then(|v| v.as_str().map(|v| v.to_owned()));
        let item2_str = data
            .get(key2)
            .and_then(|v| v.as_str().map(|v| v.to_owned()));
        match (item1_str, item2_str) {
            (Some(item1_str), Some(item2_str)) => {
                data.insert(
                    key1.to_owned(),
                    ScrapedData::Str(
                        item1_str
                            .strip_suffix(&item2_str)
                            .unwrap_or(&item1_str)
                            .trim()
                            .to_owned(),
                    ),
                );
            }
            _ => {
                if let Some(item1) = item1 {
                    data.insert(key1.to_owned(), item1);
                }
            }
        }
    }
}
