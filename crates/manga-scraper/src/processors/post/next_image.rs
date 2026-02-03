use std::collections::{BTreeMap, HashMap};

use scraper_module::ScrapedData;

use crate::processors::{PostMetadataProcessor, PostSearchProcessor};

#[derive(Default)]
pub struct NextImageProcessor;

impl PostSearchProcessor for NextImageProcessor {
    fn name(&self) -> &str {
        "next_image"
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
                    let img = cover.split_once("url=");
                    if let Some((_, img)) = img {
                        let img = img.split_once("&").map(|v| v.0).unwrap_or(img).to_owned();
                        v.cover = Some(
                            urlencoding::decode(&img)
                                .map(|v| v.to_string())
                                .unwrap_or(img),
                        );
                    }
                }
                v
            })
            .collect()
    }
}

impl PostMetadataProcessor for NextImageProcessor {
    fn name(&self) -> &str {
        "next_image"
    }

    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str) {
        let mut keys = target.split(" ");
        let key1 = keys.next().unwrap().trim();
        let item1 = data.remove(key1);
        let item1_str = item1.clone().and_then(|v| v.as_str().map(|v| v.to_owned()));
        match item1_str {
            Some(item1_str) => {
                let img = item1_str.split_once("url=");
                if let Some((_, img)) = img {
                    let img = img.split_once("&").map(|v| v.0).unwrap_or(img).to_owned();

                    data.insert(
                        key1.to_owned(),
                        ScrapedData::Str(
                            urlencoding::decode(&img)
                                .map(|v| v.to_string())
                                .unwrap_or(img),
                        ),
                    );
                    return;
                }
            }
            None => {}
        }
        if let Some(item1) = item1 {
            data.insert(key1.to_owned(), item1);
        }
    }
}
