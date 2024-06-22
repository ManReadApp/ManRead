use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReaderPageResponse {
    pub version_id: String,
    pub hide_top: f64,
    pub hide_bottom: f64,
    pub pages: HashMap<u32, Arc<ReaderPage>>,
}

impl ReaderPageResponse {
    pub fn get_page(&self, page: i32) -> Action {
        if page < 1 {
            Action::Prev
        } else if let Some(v) = self.pages.get(&(page as u32)) {
            Action::Page(v.clone())
        } else {
            Action::Next
        }
    }
}
