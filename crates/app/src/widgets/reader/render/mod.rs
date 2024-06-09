use crate::widgets::reader::overlay::render_overlay;
use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::render::helper::{get_img, get_img_};
use crate::widgets::reader::settings::ReadingMode;
use crate::widgets::reader::storage::{get_page_resp, ImageStorage, PageData, State};
use api_structure::reader::{Action, MangaReaderResponse, ReaderPage};
use eframe::emath::{pos2, vec2, Rect, Vec2};
use egui::{Color32, Ui};
use std::sync::Arc;

mod helper;
pub mod overlay;

pub fn display_images(
    v: Arc<MangaReaderResponse>,
    hierachy: &[String],
    page_data: &mut PageData,
    progress: &Progress,
    ui: &mut Ui,
    rm: &ReadingMode,
    imgs: &mut ImageStorage,
    size: Vec2,
    view_start: Vec2,
) {
    let mut chapter = progress.chapter.clone();
    let mut ch = get_page_resp(v.clone(), hierachy, page_data, &chapter, ui.ctx());
    match rm {
        ReadingMode::Single => {
            let (r, img) = get_img_(imgs, ch, progress.image).unwrap();
            let (size, gaps, scale) = max(vec2(r.width as f32, r.height as f32), size);
            let start = (gaps * 0.5) + view_start;
            img.0
                .paint_at(ui, Rect::from_min_size(start.to_pos2(), size));
            render_overlay(&img.1, start, scale, ui)
        }
        ReadingMode::Double(lr) => {
            let (r1, mut img1) = get_img_(imgs, ch.clone(), progress.image).unwrap();
            if let Some((r2, mut img2)) = get_img_(imgs, ch, progress.image + 1) {
                let new_size = vec2(size.x / 2., size.y);
                let (size1, gaps1, scale1) = max(vec2(r1.width as f32, r1.height as f32), new_size);
                let (size2, gaps2, scale2) = max(vec2(r2.width as f32, r2.height as f32), new_size);
                let start1 = vec2(gaps1.x, gaps1.y / 2.) + view_start;
                let start2 = view_start + vec2(size.x / 2., gaps2.y / 2.);
                if !lr {
                    let temp = img1;
                    img1 = img2;
                    img2 = temp;
                }
                img1.0
                    .paint_at(ui, Rect::from_min_size(start1.to_pos2(), size1));
                img2.0
                    .paint_at(ui, Rect::from_min_size(start2.to_pos2(), size2));
                render_overlay(&img1.1, start1, scale1, ui);
                render_overlay(&img2.1, start2, scale2, ui)
            } else {
                let (size, gaps, scale) = max(vec2(r1.width as f32, r1.height as f32), size);
                let start = (gaps * 0.5) + view_start;
                img1.0
                    .paint_at(ui, Rect::from_min_size(start.to_pos2(), size));
                render_overlay(&img1.1, start, scale, ui)
            }
        }
        ReadingMode::Strip => {
            let mut page = progress.image;
            let rp = Arc::new(ReaderPage::new(100, 100));
            let mut processed: f32 = -progress.pixels;
            let end: f32 = size.y;
            loop {
                let page = match &ch {
                    State::ReaderPageResponse(v_) => match v_.get_page(page as i32) {
                        Action::Page(r) => {
                            let img = get_img(&r.page_id, imgs);
                            page += 1;
                            Some((false, r, img))
                        }
                        Action::Next => {
                            let cid = v.get_next_chapter(&chapter).map(|v| v.chapter_id.clone());
                            match cid {
                                None => {
                                    break;
                                }
                                Some(v) => chapter = v,
                            }
                            ch = get_page_resp(v.clone(), hierachy, page_data, &chapter, ui.ctx());
                            page = 1;
                            ui.ctx().request_repaint();
                            None
                        }
                        _ => unreachable!(),
                    },
                    State::ChapterLoading => Some((true, rp.clone(), imgs.loading.clone())),
                    State::ChapterError => Some((true, rp.clone(), imgs.error.clone())),
                    State::NoChapter => Some((true, rp.clone(), imgs.error.clone())),
                };
                if let Some((endd, rp, img)) = page {
                    let height = rp.height(size.x);
                    let rect =
                        Rect::from_min_size(pos2(view_start.x, processed), vec2(size.x, height));
                    img.0.paint_at(ui, rect);
                    match &ch {
                        State::ReaderPageResponse(v) => {
                            let page = v.pages.iter().find_map(|(index, value)| {
                                match value.page_id == rp.page_id {
                                    true => Some(*index),
                                    false => None,
                                }
                            });
                            ui.painter().debug_rect(
                                rect,
                                Color32::RED,
                                format!(
                                    "{} {}/{}",
                                    chapter,
                                    page.unwrap_or_default(),
                                    v.pages.len()
                                ),
                            );
                        }
                        _ => {}
                    }

                    processed += height;
                    if processed >= end {
                        break;
                    }
                    if endd {
                        break;
                    }
                }
            }
        }
        ReadingMode::Row(_) => todo!("implement row reader"),
    }
}

///img_size, img_padding, scale
fn max(img: Vec2, area: Vec2) -> (Vec2, Vec2, f32) {
    let scale_x = area.x / img.x;
    let scale_y = area.y / img.y;
    let min_scale = scale_x.min(scale_y);
    let img_area = vec2(img.x * min_scale, img.y * min_scale);
    (img_area, area - img_area, min_scale)
}
