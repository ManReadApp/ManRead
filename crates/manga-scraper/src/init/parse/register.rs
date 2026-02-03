use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    sync::Arc,
};

use regex::bytes::Regex;
use scraper_module::Register;

pub struct InterpretedRegister {
    urls: Vec<UrlFilter>,
    pub processors: Vec<String>,
    img_path: PathBuf,
    extension: String,
    img_source: Option<String>,
}

enum UrlFilter {
    Regex(Regex),
    Contains(String),
    StartsWith(String),
}

impl UrlFilter {
    fn matches(&self, url: &str) -> bool {
        match self {
            UrlFilter::Regex(regex) => regex.is_match(url.as_bytes()),
            UrlFilter::StartsWith(starts_with) => url.starts_with(starts_with),
            UrlFilter::Contains(con) => url.contains(con),
        }
    }
}

impl Register for InterpretedRegister {
    fn url_matches(&self, url: &str) -> bool {
        self.urls.iter().any(|v| v.matches(url))
    }

    fn icon(&self) -> (String, Vec<u8>) {
        let file_bytes = fs::read(&self.img_path).unwrap();
        (self.extension.clone(), file_bytes)
    }

    fn get_used_processor_names(&self) -> Vec<&str> {
        self.processors.iter().map(|v| v.as_str()).collect()
    }

    fn icon_source(&self) -> Option<String> {
        self.img_source.clone()
    }
}

pub fn parse(
    file: &Path,
    uri: &str,
    img_map: &mut HashMap<String, (&str, PathBuf)>,
) -> crate::Result<Arc<dyn Register>> {
    let text = read_to_string(file)?;
    let prefixes = ["regex ", "starts_with ", "use ", "contains ", "source "];
    let mut items = text
        .lines()
        .filter_map(|v| {
            let prefix = prefixes.into_iter().find(|prefix| v.starts_with(prefix));
            prefix.map(|pre| (pre, v.strip_prefix(pre).unwrap()))
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<&str, Vec<&str>>, (pre, v)| {
                acc.entry(pre.strip_suffix(" ").unwrap())
                    .or_default()
                    .push(v);
                acc
            },
        );
    let processors = items
        .remove("use")
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();
    let mut starts_with = items
        .remove("starts_with")
        .unwrap_or_default()
        .into_iter()
        .map(|v| UrlFilter::StartsWith(v.trim().to_owned()))
        .collect::<Vec<_>>();
    let contains = items
        .remove("contains")
        .unwrap_or_default()
        .into_iter()
        .map(|v| UrlFilter::Contains(v.trim().to_owned()))
        .collect::<Vec<_>>();
    let regex = items
        .remove("regex")
        .unwrap_or_default()
        .into_iter()
        .map(|v| Ok(UrlFilter::Regex(Regex::new(v)?)))
        .collect::<Result<Vec<_>, regex::Error>>()?;
    starts_with.extend(regex);
    starts_with.extend(contains);
    let mut source = items.remove("source").unwrap_or_default();
    let img_source = match source.is_empty() {
        true => None,
        false => Some(source.remove(0).to_owned()),
    };
    let img = img_map
        .remove(uri)
        .ok_or(crate::InitError::InitParseError(format!(
            "couldnt find icon: {uri}"
        )))?;
    Ok(Arc::new(InterpretedRegister {
        urls: starts_with,
        processors,
        extension: img.0.to_owned(),
        img_path: img.1,
        img_source,
    }))
}
