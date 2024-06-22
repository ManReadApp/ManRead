use crate::error::{ApiErr, ApiErrorType};
use crate::models::manga::search::Field;
use crate::models::manga::status::Status;
use crate::search::DisplaySearch;
use crate::RequestImpl;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ExternalSearchRequest {
    pub data: ExternalSearchData,
    pub uri: String,
}

impl ExternalSearchRequest {
    pub fn next_page(&mut self) {
        match &mut self.data {
            ExternalSearchData::Simple(simple) => simple.page += 1,
            ExternalSearchData::String((_, page)) => *page += 1,
        }
    }

    pub fn reset_page(&mut self) {
        match &mut self.data {
            ExternalSearchData::Simple(simple) => simple.page = 1,
            ExternalSearchData::String((_, page)) => *page = 1,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ExternalSearchData {
    Simple(SimpleSearch),
    String((String, u32)),
}

impl ExternalSearchData {
    pub fn update_query(&mut self, new: &str) {
        match self {
            Self::Simple(simple) => {
                simple.search = new.to_string();
            }
            Self::String((query, _)) => {
                *query = new.to_string();
            }
        }
    }
    pub fn get_simple(self) -> Result<SimpleSearch, ApiErr> {
        match self {
            Self::Simple(s) => Ok(s),
            _ => Err(ApiErr {
                message: Some("wrong ExternalSearchData type".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }),
        }
    }

    pub fn get_query(self) -> (String, u32) {
        match self {
            Self::Simple(s) => (s.search, s.page),
            Self::String(s) => s,
        }
    }
}

impl RequestImpl for ExternalSearchRequest {
    const ROUTE: &'static str = "external/search";
    const AUTH: bool = true;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrapeSearchResult {
    pub title: String,
    pub url: String,
    pub cover: String,
    pub r#type: Option<String>,
    pub status: Option<String>,
}

impl DisplaySearch for ScrapeSearchResult {
    fn image_number(&self) -> u32 {
        0
    }

    fn internal(&self) -> bool {
        false
    }

    fn id_url(&self) -> &String {
        &self.url
    }

    fn ext(&self) -> Cow<String> {
        Cow::Owned("".to_string())
    }

    fn status(&self) -> Cow<Status> {
        Cow::Owned(Status::Ongoing)
    }

    fn titles(&self) -> Cow<HashMap<String, Vec<String>>> {
        let mut hm = HashMap::new();
        hm.insert("eng".to_string(), vec![self.title.clone()]);
        Cow::Owned(hm)
    }

    fn cover(&self) -> &str {
        &self.cover
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValidSearch {
    pub sorts: Vec<String>,
    pub tags: Vec<String>,
    pub status: Vec<String>,
}

impl ValidSearch {
    pub fn anilist() -> Self {
        Self {
            sorts: vec![
                "popularity".to_string(),
                "score".to_string(),
                "trending".to_string(),
                "created".to_string(),
                "updated".to_string(),
            ],
            tags: vec![],
            status: vec![
                "releasing".to_string(),
                "finished".to_string(),
                "hiatus".to_string(),
                "cancelled".to_string(),
                "upcoming".to_string(),
            ],
        }
    }

    pub fn kitsu() -> Self {
        Self {
            sorts: vec![
                "popularity".to_string(),
                "rating".to_string(),
                "updated".to_string(),
                "created".to_string(),
            ],
            tags: vec![],
            status: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimpleSearch {
    pub search: String,
    pub sort: Option<String>,
    pub desc: bool,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub page: u32,
}

impl SimpleSearch {
    pub fn validate(&self, vs: &ValidSearch) -> bool {
        if let Some(v) = &self.sort {
            if !vs.sorts.contains(v) {
                return false;
            }
        }
        if let Some(v) = &self.status {
            if !vs.status.contains(v) {
                return false;
            }
        }
        for tag in &self.tags {
            if !vs.tags.contains(tag) {
                //TODO:
                //return false;
            }
        }
        true
    }
}

#[derive(Deserialize, Serialize)]
pub enum ValidSearches {
    String,
    ValidSearch(ValidSearch),
}

impl ValidSearches {
    pub fn parser(&self) -> Option<Vec<Field>> {
        None
    }
}
