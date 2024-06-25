use crate::error::ScrapeError;
use crate::extractor::parser::Field;
use crate::extractor::SearchServiceDeserialized;
use crate::services::metadata::MetaDataService;
use crate::services::multisite::MultiSiteService;
use crate::services::search::SearchService;
use crate::services::singlesite::SingleSiteService;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io;
use std::io::{read_to_string, BufRead};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod icon;
pub mod metadata;
pub mod multisite;
pub mod search;
pub mod singlesite;

pub struct Service {
    fields: Vec<Field>,
    config: HashMap<String, String>,
}

impl Service {
    fn process(&self, html: &str) -> HashMap<String, String> {
        let html = html.replace("<noscript>", "").replace("</noscript>", "");
        self.fields
            .iter()
            .filter_map(|v| {
                v.get(&html).map(|res| (v.name.clone(), res))
            })
            .collect::<HashMap<_, _>>()
    }
}

pub fn init(
    root_folder: PathBuf,
) -> Result<
    (
        MultiSiteService,
        SingleSiteService,
        SearchService,
        MetaDataService,
    ),
    ScrapeError,
> {
    let folder = root_folder.join("external");
    let mut search = HashMap::new();
    let mut meta = HashMap::new();
    let mut multi = HashMap::new();
    let mut single = HashMap::new();
    for entry in read_dir(&folder)? {
        let path = entry?.path();
        if path.is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if !name.starts_with('.') {
                if let Some(scraper) = name.strip_suffix(".scraper") {
                    let (service, kind) = get_services(&folder, &path)?;
                    match kind {
                        None => panic!(),
                        Some(v) => {
                            match v {
                                Kind::SingleSiteScraper => {
                                    single.insert(scraper.to_string(), service)
                                }
                                Kind::MultiSiteScraper => {
                                    multi.insert(scraper.to_string(), service)
                                }
                            };
                        }
                    }
                } else if let Some(metadata) = name.strip_suffix(".metadata") {
                    meta.insert(metadata.to_string(), get_services(&folder, &path)?.0);
                } else if let Some(v) = name.strip_suffix(".search") {
                    let file = File::open(path.as_path())?;
                    let str = read_to_string(file)?;
                    let data: SearchServiceDeserialized = serde_json::from_str(&str)?;
                    search.insert(v.to_string(), data.convert(&folder));
                }
            }
        }
    }
    Ok((
        MultiSiteService::new(multi),
        SingleSiteService::new(single),
        SearchService::new(search),
        MetaDataService::new(meta),
    ))
}

fn get_services(folder: &Path, path: &Path) -> Result<(Service, Option<Kind>), ScrapeError> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();
    if let Some(Ok(first_line)) = lines.next() {
        let header: Header = serde_json::from_str(&format!("{}{}{}", '{', first_line, '}'))?;
        let text = lines
            .collect::<Result<Vec<String>, _>>()
            .unwrap()
            .join("\n");
        let v = Field::parse(text.as_str());
        let config = if let Some(file) = header.request_config {
            let text = read_to_string(File::open(folder.join(file)).unwrap())?;
            serde_json::from_str(&text)?
        } else {
            HashMap::new()
        };
        Ok((Service { fields: v, config }, header.kind))
    } else {
        Err(ScrapeError::input_error(format!(
            "header missing in file: {}",
            path.display()
        )))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Header {
    kind: Option<Kind>,
    request_config: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Kind {
    SingleSiteScraper,
    MultiSiteScraper,
}

pub fn config_to_request_builder(
    client: &Client,
    config: &HashMap<String, String>,
    url: &str,
) -> RequestBuilder {
    let method = config.get("METHOD").cloned().unwrap_or("GET".to_string());
    let headers = config
        .iter()
        .map(|(key, value)| {
            (
                HeaderName::from_str(key).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            )
        })
        .collect();
    client
        .request(Method::from_str(method.as_str()).unwrap(), url)
        .headers(headers)
}
