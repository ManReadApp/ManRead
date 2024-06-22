use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TranslationArea {
    pub translated_text: HashMap<String, String>,
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub text_color: [u8; 3],
    pub outline_color: [u8; 3],
    pub background: String,
}
