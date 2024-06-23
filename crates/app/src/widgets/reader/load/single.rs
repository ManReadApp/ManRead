use crate::widgets::reader::load::load_image;
use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::storage::{ImageStorage, PageData};
use api_structure::resp::reader::pages::ReaderPageResponse;
use api_structure::resp::reader::MangaReaderResponse;
use egui::Context;
use std::sync::Arc;

pub fn single(
    v: &Arc<MangaReaderResponse>,
    hierachy: &[String],
    chapter: &str,
    area: f32,
    progress: &Progress,
    reader_page: &Arc<ReaderPageResponse>,
    i_: u32,
    imgs_: &mut ImageStorage,
    ctx: &Context,
    page_data: &mut PageData,
) {
    let area = area as u32 * i_;
    for i in 0..=area + i_ {
        if load_image(
            v,
            hierachy,
            chapter,
            imgs_,
            reader_page.clone(),
            ctx,
            page_data,
            progress.image as i32 + i as i32,
        )
        .0
        {
            break;
        }
        if i != 0
            && i <= area
            && load_image(
                v,
                hierachy,
                chapter,
                imgs_,
                reader_page.clone(),
                ctx,
                page_data,
                progress.image as i32 - i as i32,
            )
            .0
        {
            break;
        }
    }
}
