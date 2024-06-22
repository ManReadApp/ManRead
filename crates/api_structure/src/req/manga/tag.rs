use serde::{Deserialize, Serialize};

use crate::models::manga::tag::TagSex;

#[derive(Deserialize, Serialize)]
pub struct TagsRequest {
    pub query: String,
    pub limit: usize,
    pub sex: TagSex,
}
