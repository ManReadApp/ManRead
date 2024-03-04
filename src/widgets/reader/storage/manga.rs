use crate::fetcher::{Complete, Fetcher};
use crate::get_app_data;
use crate::widgets::reader::storage::{PageData, State};
use api_structure::reader::{MangaReaderResponse, ReaderChapter, ReaderPageRequest};
use api_structure::RequestImpl;
use egui::Context;
use std::sync::Arc;

pub fn get_page_resp(
    v: Arc<MangaReaderResponse>,
    hierachy: &[String],
    page_data: &mut PageData,
    chapter: &str,
    ctx: &Context,
) -> State {
    let ver = get_version(v.get_chapter(chapter).unwrap(), hierachy);
    if let Some(ver) = ver {
        if let Some(v) = page_data.get_mut(ver) {
            return if let Some(v) = v.result() {
                match v {
                    Complete::Json(cv) => State::ReaderPageResponse(cv.clone()),
                    _ => State::ChapterError,
                }
            } else {
                State::ChapterLoading
            };
        }
        let mut fetcher = Fetcher::new(ReaderPageRequest::request(&get_app_data().url).unwrap());
        fetcher.set_ctx(ctx.clone());
        fetcher.set_body(&ReaderPageRequest {
            chapter_version_id: ver.to_string(),
        });
        fetcher.send();
        page_data.insert(ver.to_string(), fetcher);
        return get_page_resp(v, hierachy, page_data, chapter, ctx);
    }
    State::NoChapter
}

/// gets prioritized  version
pub fn get_version<'a>(rc: &'a ReaderChapter, hierachy: &[String]) -> Option<&'a String> {
    let key = rc.versions.keys().next();
    if key.is_none() {
        return None;
    }
    for id in hierachy {
        if let Some(v) = rc.versions.get(id) {
            return Some(v);
        }
    }
    rc.versions.get(key.unwrap())
}

pub fn get_version_key(rc: &ReaderChapter, hierachy: &[String]) -> Option<String> {
    let key = rc.versions.keys().next();
    if key.is_none() {
        return None;
    }
    for id in hierachy {
        return Some(id.clone());
    }
    return key.cloned();
}
