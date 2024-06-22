use std::collections::HashMap;

use crate::models::manga::external_search::ValidSearches;

pub mod manga;
pub mod reader;

pub type ByteResponse = Vec<u8>;
pub type NoResponse = Vec<u8>;

pub type FontsResponse = Vec<String>;
pub type AvailableExternalSitesResponse = HashMap<String, ValidSearches>;
