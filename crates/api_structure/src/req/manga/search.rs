use serde::{Deserialize, Serialize};

use crate::models::manga::search::{Array, ItemOrArray, Order};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    pub order: String,
    pub desc: bool,
    pub limit: u32,
    pub page: u32,
    pub query: Array,
}
