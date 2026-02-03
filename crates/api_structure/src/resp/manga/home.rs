use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::search::SearchResponse;

#[derive(Serialize, Deserialize, Debug, ApiComponent, JsonSchema)]
pub struct HomeResponse {
    pub trending: Vec<SearchResponse>,
    pub newest: Vec<SearchResponse>,
    pub latest_updates: Vec<SearchResponse>,
    pub favorites: Vec<SearchResponse>,
    pub reading: Vec<SearchResponse>,
    pub random: Vec<SearchResponse>,
}
