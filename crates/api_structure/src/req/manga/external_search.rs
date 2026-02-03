use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::manga::external_search::ExternalSearchData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalSearchRequest {
    pub data: ExternalSearchData,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemOrArrayOrMap {
    Item(String),
    Array(Vec<String>),
    Map(HashMap<String, String>),
    ArrayDyn(Vec<Value>),
}

#[derive(Serialize, Deserialize)]
pub struct ExternalInfoRequest {
    pub url: String,
}

impl ExternalSearchRequest {
    pub fn next_page(&mut self) {
        match &mut self.data {
            ExternalSearchData::Simple(simple) => simple.page += 1,
            ExternalSearchData::String((_, page)) => *page += 1,
            ExternalSearchData::Advanced(v) => v.page += 1,
        }
    }

    pub fn reset_page(&mut self) {
        match &mut self.data {
            ExternalSearchData::Simple(simple) => simple.page = 1,
            ExternalSearchData::String((_, page)) => *page = 1,
            ExternalSearchData::Advanced(v) => v.page = 1,
        }
    }
}
