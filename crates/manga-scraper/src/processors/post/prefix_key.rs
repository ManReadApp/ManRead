use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

#[derive(Default)]
pub struct PrefixKeyProcessor;

fn split(v: &ScrapedData, key2: &str) -> ScrapedData {
    match v {
        ScrapedData::Str(s) => match s.split_once(key2) {
            Some((k, v)) => ScrapedData::Map(
                vec![(k.trim().to_owned(), ScrapedData::Str(v.trim().to_owned()))]
                    .into_iter()
                    .collect(),
            ),
            None => ScrapedData::Str(s.clone()),
        },
        ScrapedData::Arr(scraped_datas) => {
            let v = scraped_datas
                .iter()
                .filter_map(|v| split(v, key2).as_map())
                .flatten()
                .collect::<BTreeMap<_, _>>();
            if v.len() == scraped_datas.len() {
                ScrapedData::Map(v)
            } else {
                ScrapedData::Arr(scraped_datas.clone())
            }
        }
        v => v.clone(),
    }
}
impl PostMetadataProcessor for PrefixKeyProcessor {
    fn name(&self) -> &str {
        "prefix_key"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next().unwrap().trim();
        let key2 = keys.next().unwrap().trim();
        println!("{} {}", key1, key2);
        if let Some(v) = data.remove(key1) {
            let new = split(&v, key2);
            data.insert(key1.to_owned(), new);
        }
    }
}
