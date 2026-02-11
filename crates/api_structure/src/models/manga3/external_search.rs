use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{search::DisplaySearch, v1::Status};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrapeSearchResponse {
    pub title: String,
    pub url: String,
    pub cover: String,
    pub r#type: Option<String>,
    pub status: Option<String>,
}

impl DisplaySearch for ScrapeSearchResponse {
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
