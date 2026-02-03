use std::collections::BTreeMap;

use scraper_module::ScrapedData;

use crate::processors::PostMetadataProcessor;

#[derive(Default)]
pub struct StripSuffixStringProcessor;

impl PostMetadataProcessor for StripSuffixStringProcessor {
    fn name(&self) -> &str {
        "strip_suffix_str"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next().unwrap().trim();
        let suffix = keys.next().unwrap().trim();
        let item1 = data.remove(key1);
        if let Some(v) = item1 {
            data.insert(key1.to_owned(), strip_suffix(v, suffix));
        }
    }
}

fn strip_suffix(d: ScrapedData, suffix: &str) -> ScrapedData {
    match d {
        ScrapedData::Str(v) => {
            if let Some(new) = v.strip_suffix(suffix) {
                ScrapedData::Str(new.trim().to_owned())
            } else {
                ScrapedData::Str(v)
            }
        }
        ScrapedData::Arr(vec) => {
            ScrapedData::Arr(vec.into_iter().map(|v| strip_suffix(v, suffix)).collect())
        }
        ScrapedData::Map(hash_map) => {
            let mut hm = BTreeMap::new();
            for (k, v) in hash_map {
                hm.insert(
                    k.strip_suffix(suffix).unwrap_or(&k).trim().to_owned(),
                    strip_suffix(v, suffix),
                );
            }
            ScrapedData::Map(hm)
        }
        ScrapedData::Map2(hash_map) => {
            let mut out = vec![];
            for (k, v) in hash_map {
                out.push((
                    k.strip_suffix(suffix).unwrap_or(&k).trim().to_owned(),
                    strip_suffix(v, suffix),
                ));
            }
            ScrapedData::Map2(out)
        }
    }
}
