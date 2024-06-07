use crate::RequestImpl;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
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
        missing.into_iter().map(|v| v.parse().unwrap()).collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReaderChapter {
    pub chapter_id: String,
    pub titles: Vec<String>,
    pub chapter: f64,
    pub sources: Vec<String>,
    pub release_date: Option<String>,
    ///Version, versionchapter
    pub versions: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct MangaReaderRequest {
    pub manga_id: String,
    pub chapter_id: Option<String>,
}

impl RequestImpl for MangaReaderRequest {
    const ROUTE: &'static str = "reader_info";
    const AUTH: bool = true;
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

#[derive(Serialize, Deserialize)]
pub struct ReaderPageRequest {
    pub chapter_version_id: String,
}

impl RequestImpl for ReaderPageRequest {
    const ROUTE: &'static str = "pages";
    const AUTH: bool = true;
}

#[derive(Serialize, Deserialize)]
pub struct ReaderPageResponse {
    pub version_id: String,
    pub hide_top: f64,
    pub hide_bottom: f64,
    pub pages: HashMap<u32, Arc<ReaderPage>>,
}

pub enum Action {
    Prev,
    Page(Arc<ReaderPage>),
    Next,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Progress {
    pub width_start: f64,
    pub width_end: f64,
    pub height_start: f64,
    pub height_end: f64,
}

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
