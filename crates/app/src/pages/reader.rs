use crate::fetcher::{Complete, Fetcher};
use crate::get_app_data;
use crate::widgets::reader::load::load_images;
use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::render::display_images;
use crate::widgets::reader::render::overlay::Overlay;
use crate::widgets::reader::scroll::set_progress;
use crate::widgets::reader::settings::{get_screen_dim, ReadingMode, Settings, ViewArea};
use crate::widgets::reader::storage::{get_page_resp, State, Storage};
use api_structure::reader::MangaReaderRequest;
use api_structure::RequestImpl;
use eframe::{App, Frame};
use egui::{pos2, vec2, Color32, Context, Rect, Rounding};

pub struct MangaReaderPage {
    storage: Storage,
    settings: Settings,
    progress: Option<Progress>,
    init: bool,
}

impl MangaReaderPage {
    pub(crate) fn new(manga_id: String, chapter_id: Option<String>) -> Self {
        let mut manga = Fetcher::new(MangaReaderRequest::request(&get_app_data().url).unwrap());
        manga.set_body(MangaReaderRequest {
            manga_id,
            chapter_id,
        });
        manga.send();
        Self {
            storage: Storage {
                manga,
                page_data: Default::default(),
                loaded_pages: Default::default(),
            },
            settings: Settings {
                version_hierachy: vec![],
                reading_mode: ReadingMode::Strip,
                prefetch: 3.0,
                view_area: ViewArea {
                    margin_top: 0.0,
                    margin_right: 0.0,
                    margin_bottom: 0.0,
                    margin_left: 0.0,
                },
            },
            progress: None,
            init: false,
        }
    }
}

impl App for MangaReaderPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if !self.init {
            self.init = true;
            self.storage.manga.set_ctx(ctx.clone());
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(v) = self.storage.manga.result().cloned() {
                match v {
                    Complete::Json(v) => {
                        let size = self.settings.view_area.get_size(ctx);
                        if let Some(p) = &mut self.progress {
                            set_progress(
                                ui,
                                &self.settings.reading_mode,
                                p,
                                v.clone(),
                                &self.settings.version_hierachy,
                                &mut self.storage.page_data,
                                size,
                            );
                            load_images(
                                v.clone(),
                                &self.settings.version_hierachy,
                                &mut self.storage.page_data,
                                &p.chapter,
                                ctx,
                                &self.settings.reading_mode,
                                self.settings.prefetch,
                                p,
                                size,
                                &mut self.storage.loaded_pages,
                            );
                            display_images(
                                v.clone(),
                                &self.settings.version_hierachy,
                                &mut self.storage.page_data,
                                p,
                                ui,
                                &self.settings.reading_mode,
                                &mut self.storage.loaded_pages,
                                size,
                                vec2(
                                    self.settings.view_area.margin_left,
                                    self.settings.view_area.margin_top,
                                ),
                            );
                            let painter = ui.painter();
                            let dim = get_screen_dim(ctx);
                            let height = 50.;
                            let top = Rect::from_min_size(pos2(0., 0.), vec2(dim.x, height));
                            painter.rect_filled(
                                Rect::from_min_size(pos2(0., dim.y - height), vec2(dim.x, height)),
                                Rounding::ZERO,
                                Color32::from_rgba_unmultiplied(0, 0, 0, (0.7 * 255.) as u8),
                            );
                            ui.put(top, Overlay::new("One Piece", 1., None));
                            self.storage.loaded_pages.clean(ctx);
                        } else if v.no_chapters() {
                            //TODO: no chapters
                        } else {
                            let chapter = get_page_resp(
                                v.clone(),
                                &self.settings.version_hierachy,
                                &mut self.storage.page_data,
                                &v.open_chapter,
                                ctx,
                            );
                            let progress = v.progress;
                            match chapter {
                                State::ChapterLoading => {
                                    //TODO: display spinner
                                }
                                State::ChapterError => todo!("display error"),
                                State::ReaderPageResponse(rp) => {
                                    let (page, pixels) = rp
                                        .pages
                                        .iter()
                                        .find_map(|(page, data)| {
                                            match progress >= data.progress.height_start
                                                && progress <= data.progress.height_end
                                            {
                                                true => {
                                                    let gap = data.progress.height_end
                                                        - data.progress.height_start;
                                                    let pro = progress - data.progress.height_start;
                                                    let progress_in_image = pro / gap;
                                                    let progress = data.height(size.x) as f64
                                                        * progress_in_image;
                                                    Some((*page, progress as f32))
                                                }
                                                false => None,
                                            }
                                        })
                                        .unwrap_or((
                                            rp.pages.keys().max().copied().unwrap_or(1),
                                            0.0,
                                        ));
                                    self.progress = Some(Progress {
                                        chapter: v.open_chapter.to_string(),
                                        image: page,
                                        pixels,
                                    });
                                }
                                State::NoChapter => todo!("display error"),
                            }
                        }
                    }
                    _ => v.display_error(ui),
                }
            }
        });
    }
}
