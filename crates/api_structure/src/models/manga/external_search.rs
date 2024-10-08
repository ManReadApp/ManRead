use serde::{Deserialize, Serialize};

use super::search::Field;
use crate::error::{ApiErr, ApiErrorType};
use crate::req::manga::search::SearchRequest;

#[derive(Serialize, Deserialize)]
pub enum ExternalSearchData {
    Advanced(SearchRequest),
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
            ExternalSearchData::Advanced(v) => {
                todo!()
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
            ExternalSearchData::Advanced(v) => ("".to_string(), v.page),
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
