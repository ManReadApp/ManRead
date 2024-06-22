use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub enum Action {
    Prev,
    Page(Arc<ReaderPage>),
    Next,
}

#[derive(Serialize, Deserialize)]
pub struct ReaderPage {
    pub page_id: String,
    pub width: u32,
    pub height: u32,
    pub ext: String,
    pub translation: bool,
    pub progress: Progress,
}

impl ReaderPage {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            page_id: "".to_string(),
            width: w,
            height: h,
            ext: "gif".to_string(),
            translation: false,
            progress: Progress {
                width_start: 0.0,
                width_end: 0.0,
                height_start: 0.0,
                height_end: 0.0,
            },
        }
    }
    pub fn width(&self, available_height: f32) -> f32 {
        (available_height / self.height as f32) * self.width as f32
    }
    pub fn height(&self, available_width: f32) -> f32 {
        (available_width / self.width as f32) * self.height as f32
    }
}
