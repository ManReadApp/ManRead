use crate::error::ScrapeError;
use api_structure::error::{ApiErr, ApiErrorType};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

enum Filter {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    Regex(Regex),
}

pub struct ExternalSite {
    filters: Vec<Filter>,
    pub path_buf: PathBuf,
    pub uri: String,
}

impl ExternalSite {
    pub fn init(root_folder: PathBuf) -> Result<Vec<Self>, String> {
        let mut files = HashMap::new();
        let mut filters = HashMap::new();
        for dir in read_dir(root_folder.join("external")).map_err(|e| e.to_string())? {
            let dir = dir.map_err(|e| e.to_string())?;
            let path = dir.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                if let Some((name, ext)) = name.split_once('.') {
                    match ext {
                        "filter" => {
                            filters.insert(
                                name.to_string(),
                                Filter::new(
                                    read_to_string(File::open(path).unwrap())
                                        .map_err(|e| e.to_string())?,
                                )?,
                            );
                        }
                        "scraper" | "search" | "metadata" => {}
                        _ => {
                            files.insert(name.to_string(), path);
                        }
                    }
                }
            }
        }

        Ok(filters
            .into_iter()
            .map(|(site, filter)| ExternalSite {
                filters: filter,
                path_buf: files
                    .get(&site)
                    .ok_or("Failed to find file".to_string())
                    .unwrap()
                    .clone(),
                uri: site,
            })
            .collect())
    }

    pub fn check(&self, url: &str) -> bool {
        self.filters.iter().any(|v| v.check(url))
    }
}

impl Filter {
    pub fn new(value: String) -> Result<Vec<Self>, String> {
        value
            .split('\n')
            .flat_map(|v| {
                if let Some(v) = v.strip_prefix("starts_with ") {
                    Some(Ok(Filter::StartsWith(v.to_string())))
                } else if let Some(v) = v.strip_prefix("contains ") {
                    Some(Ok(Filter::Contains(v.to_string())))
                } else if let Some(v) = v.strip_prefix("regex ") {
                    let regex = Regex::from_str(v);
                    match regex {
                        Ok(r) => Some(Ok(Filter::Regex(r))),
                        Err(e) => Some(Err(e.to_string())),
                    }
                } else {
                    v.strip_prefix("ends_with ")
                        .map(|v| Ok(Filter::EndsWith(v.to_string())))
                }
            })
            .collect::<Result<Vec<_>, String>>()
    }

    pub fn check(&self, url: &str) -> bool {
        match self {
            Filter::StartsWith(v) => url.starts_with(v),
            Filter::EndsWith(v) => url.ends_with(v),
            Filter::Contains(v) => url.contains(v),
            Filter::Regex(v) => v.is_match(url),
        }
    }
}

pub fn get_uri(data: &Arc<Vec<ExternalSite>>, url: &str) -> Result<String, ScrapeError> {
    for external in data.iter() {
        if external.check(url) {
            return Ok(external.uri.clone());
        }
    }
    Err(ApiErr {
        message: Some("couldnt find uri".to_string()),
        cause: None,
        err_type: ApiErrorType::InternalError,
    }
    .into())
}
