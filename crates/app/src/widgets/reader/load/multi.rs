use crate::widgets::reader::load::load_image;
use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::storage::{ImageStorage, PageData};
use api_structure::reader::{Action, MangaReaderResponse, ReaderPageResponse};
use egui::Context;
use std::sync::Arc;

pub fn multi(
    v: &Arc<MangaReaderResponse>,
    hierachy: &[String],
    chapter: &str,
    area: f32,
    progress: &Progress,
    reader_page: &Arc<ReaderPageResponse>,
    x: f32,
    y: f32,
    height: bool,
    imgs_: &mut ImageStorage,
    ctx: &Context,
    page_data: &mut PageData,
) {
    let area = y * area;
    let mut processed = 0.;
    let mut count = 0;
    while processed < area + y {
        let (br, img) = load_image(
            v,
            hierachy,
            chapter,
            imgs_,
            reader_page.clone(),
            ctx,
            page_data,
            progress.image as i32 + count,
        );
        if let Action::Page(v) = img {
            let a = if height { v.height(x) } else { v.width(x) };
            processed += a;
        }
        if br {
            break;
        }
        count += 1;
    }

    count = 1;
    processed = 0.0;
    while processed < area {
        let (br, img) = load_image(
            v,
            hierachy,
            chapter,
            imgs_,
            reader_page.clone(),
            ctx,
            page_data,
            progress.image as i32 - count,
        );
        if let Action::Page(v) = img {
            let a = if height { v.height(x) } else { v.width(x) };
            processed += a;
        }
        if br {
            break;
        }
        count += 1;
    }
}
