use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::settings::ReadingMode;
use crate::widgets::reader::storage::{get_page_resp, PageData, State};
use api_structure::models::reader::page::Action;
use api_structure::resp::reader::MangaReaderResponse;
use eframe::emath::Vec2;
use egui::Ui;
use std::sync::Arc;
fn get_scroll_delta(ui: &mut Ui) -> Vec2 {
    ui.input(|i| i.smooth_scroll_delta)
}

pub fn report_progress(progress: f64) {
    println!("{progress}")
}
pub fn set_progress(
    ui: &mut Ui,
    rm: &ReadingMode,
    progress: &mut Progress,
    mrr: Arc<MangaReaderResponse>,
    hierachy: &[String],
    page_data: &mut PageData,
    area: Vec2,
) {
    match rm {
        ReadingMode::Strip => {
            let scroll_delta = get_scroll_delta(ui);
            if scroll_delta == Vec2::ZERO {
                return;
            }

            let mut ch = get_page_resp(
                mrr.clone(),
                hierachy,
                page_data,
                &progress.chapter,
                ui.ctx(),
            );
            if let State::ReaderPageResponse(v) = &ch {
                match v.get_page(progress.image as i32) {
                    Action::Page(page) => {
                        let max = page.height(area.x);
                        let processed = progress.pixels - scroll_delta.y;
                        if processed > max {
                            match v.get_page((progress.image + 1) as i32) {
                                Action::Prev => unreachable!(),
                                Action::Page(_) => {
                                    progress.image += 1;
                                    progress.pixels = processed - max;
                                }
                                Action::Next => {
                                    if let Some(v) = mrr.get_next_chapter(&progress.chapter) {
                                        ch = get_page_resp(
                                            mrr.clone(),
                                            hierachy,
                                            page_data,
                                            &v.chapter_id,
                                            ui.ctx(),
                                        );
                                        progress.image = 1;
                                        progress.pixels = processed - max;
                                        progress.chapter.clone_from(&v.chapter_id);
                                    }
                                }
                            }
                        } else if processed < 0.0 {
                            match v.get_page(progress.image as i32 - 1) {
                                Action::Prev => {
                                    if let Some(v) = mrr.get_prev_chapter(&progress.chapter) {
                                        ch = get_page_resp(
                                            mrr.clone(),
                                            hierachy,
                                            page_data,
                                            &v.chapter_id,
                                            ui.ctx(),
                                        );
                                        if let State::ReaderPageResponse(rpp) = &ch {
                                            let last_page =
                                                rpp.pages.keys().max().copied().unwrap();
                                            progress.chapter.clone_from(&v.chapter_id);
                                            let v = rpp.pages.get(&last_page).unwrap();
                                            progress.image = last_page;
                                            progress.pixels = v.height(area.x) + processed
                                        }
                                    }
                                }
                                Action::Page(v) => {
                                    progress.image -= 1;
                                    progress.pixels = v.height(area.x) + processed;
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            progress.pixels = processed;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            if let State::ReaderPageResponse(v) = ch {
                let page = v.pages.get(&progress.image).unwrap();
                let start = page.progress.height_start;
                let gap = page.progress.height_end - start;
                let img_progress = progress.pixels as f64 / page.height(area.x) as f64;
                report_progress(start + gap * img_progress);
            }
        }
        ReadingMode::Row(_) => {}
        ReadingMode::Single => {}
        ReadingMode::Double(_) => {}
    }
}
