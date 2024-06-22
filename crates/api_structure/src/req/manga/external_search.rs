use serde::{Deserialize, Serialize};

use crate::models::manga::external_search::ExternalSearchData;

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
