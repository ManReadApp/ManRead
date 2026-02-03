pub mod pages;

use std::collections::{HashMap, HashSet};

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::reader::chapter::ReaderChapter;

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema)]
pub struct MangaReaderResponse {
    pub manga_id: String,
    pub titles: HashMap<String, Vec<String>>,
    pub kind: String,
    pub description: Option<String>,
    pub chapters: Vec<ReaderChapter>,
    pub favorite: bool,
    /// manga_id
    pub open_chapter: String,
    pub progress: f64,
}

impl MangaReaderResponse {
    pub fn get_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        self.chapters.iter().find(|&ch| ch.chapter_id == id)
    }

    pub fn get_prev_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        let mut last = None;
        for ch in &self.chapters {
            if ch.chapter_id == id {
                break;
            }
            last = Some(&ch.chapter_id)
        }
        match last {
            None => None,
            Some(v) => self.get_chapter(v),
        }
    }

    pub fn get_next_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        let mut hit = false;
        for ch in &self.chapters {
            if hit {
                return Some(ch);
            }
            if ch.chapter_id == id {
                hit = true;
            }
        }
        None
    }
}

impl MangaReaderResponse {
    pub fn no_chapters(&self) -> bool {
        self.chapters.is_empty()
    }
    pub fn missing_chapters(&self) -> Vec<f64> {
        let ch = self.chapters.iter().map(|v| v.chapter).collect::<Vec<_>>();
        let max = max_f64(&ch);

        let ch = ch
            .into_iter()
            .map(|v| v.to_string())
            .collect::<HashSet<_>>();
        let mut missing = vec![];
        if let Some(v) = max {
            for num in 1..v.floor() as u32 {
                let num = (num as f64).to_string();
                if !ch.contains(&num) {
                    missing.push(num);
                }
            }
        }
        missing
            .into_iter()
            .map(|v| v.parse().expect("cant fail. f64 => to_string => f64"))
            .collect()
    }
}

fn max_f64(items: &Vec<f64>) -> Option<f64> {
    let mut max = None;
    for item in items {
        if let Some(max) = &mut max {
            if item > max {
                *max = *item;
            }
        } else {
            max = Some(*item)
        }
    }
    max
}
