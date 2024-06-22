use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{models::manga::status::Status, search::DisplaySearch};

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResponse {
    pub manga_id: String,
    pub titles: HashMap<String, Vec<String>>,
    pub tags: Vec<String>,
    pub status: Status,
    pub ext: String,
    pub number: u32,
}

impl DisplaySearch for SearchResponse {
    fn image_number(&self) -> u32 {
        self.number
    }

    fn internal(&self) -> bool {
        true
    }

    fn id_url(&self) -> &String {
        &self.manga_id
    }

    fn ext(&self) -> Cow<String> {
        Cow::Borrowed(&self.ext)
    }

    fn status(&self) -> Cow<Status> {
        Cow::Borrowed(&self.status)
    }

    fn titles(&self) -> Cow<HashMap<String, Vec<String>>> {
        Cow::Borrowed(&self.titles)
    }

    fn cover(&self) -> &str {
        ""
    }
}
