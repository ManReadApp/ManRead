use serde::{Deserialize, Serialize};

use super::search::SearchResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct HomeResponse {
    pub trending: Vec<SearchResponse>,
    pub newest: Vec<SearchResponse>,
    pub latest_updates: Vec<SearchResponse>,
    pub favorites: Vec<SearchResponse>,
    pub reading: Vec<SearchResponse>,
    pub random: Vec<SearchResponse>,
}
