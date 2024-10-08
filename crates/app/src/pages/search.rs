use crate::fetcher::{Complete, Fetcher};
use crate::get_app_data;
use crate::requests::{
    AvailableExternalSitesRequestFetcher, ExternalSearchRequestFetcher, RequestImpl,
    SearchRequestFetcher,
};
use crate::util::new_parser::search_parser;
use crate::window_storage::{Initter, Page};
use api_structure::models::manga::external_search::ExternalSearchData;
use api_structure::models::manga::search::{Array, Field, ItemKind};
use api_structure::req::manga::external_search::ExternalSearchRequest;
use api_structure::req::manga::search::SearchRequest;
use api_structure::req::manga::AvailableExternalSitesRequest;
use api_structure::resp::manga::external_search::ScrapeSearchResponse;
use api_structure::resp::manga::search::SearchResponse;

use api_structure::search::DisplaySearch;
use eframe::emath::vec2;
use eframe::{App, Frame};
use egui::scroll_area::ScrollBarVisibility;
use egui::Response;
use egui::{
    Color32, ComboBox, Context, Grid, Label, OpenUrl, ScrollArea, Sense, Spinner, TextEdit, Ui,
};
use log::{debug, error};
use std::mem;
use std::sync::MutexGuard;

pub struct SearchPage {
    search: SearchComponent,
}

pub struct SearchData<D: DisplaySearch> {
    pub searched: Vec<D>,
    pub fetcher: Fetcher<Vec<D>>,
    pub search: String,
    pub end: bool,
    pub require_new: bool,
    pub reload: bool,
}

impl<D: DisplaySearch> SearchData<D> {
    pub fn set_load(&mut self, need: impl FnOnce() -> bool) {
        if self.end {
            self.require_new = false
        } else {
            self.require_new = need()
        }
    }
}

impl SearchPage {
    pub fn new() -> Self {
        let mut fetcher: SearchRequestFetcher = SearchRequest::fetcher(&get_app_data().url);
        fetcher.set_body(&*get_app_data().search.lock().unwrap());
        let mut search = get_app_data().search.lock().unwrap().query.to_string();
        if search.starts_with("and:(") && search.ends_with(')') {
            search = search
                .strip_prefix("and:(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
                .to_string();
        }
        let searches = AvailableExternalSitesRequest::fetcher(&get_app_data().url);
        Self {
            search: SearchComponent {
                internal: SearchData {
                    searched: vec![],
                    fetcher,
                    search,
                    end: false,
                    require_new: false,
                    reload: false,
                },
                external: SearchData {
                    searched: vec![],
                    fetcher: ExternalSearchRequest::fetcher(&get_app_data().url),
                    search: "".to_string(),
                    end: false,
                    require_new: false,
                    reload: false,
                },
                external_search: ExternalSearchRequest {
                    data: ExternalSearchData::String(("".to_string(), 1)),
                    uri: "asura".to_string(),
                },
                external_change: false,
                reset_scroll: false,
                selected_search: "internal".to_string(),
                searches,
                init: Default::default(),
            },
        }
    }
}

impl<T: DisplaySearch> SearchData<T> {
    fn move_data_external(&mut self, ctx: &Context, search: &mut ExternalSearchRequest) {
        if self.fetcher.result().is_some() {
            let mut new = ExternalSearchRequest::cfetcher_ctx(&get_app_data().url, ctx.clone());
            mem::swap(&mut new, &mut self.fetcher);
            let result = new.take_result().unwrap();
            match result {
                Complete::ApiError(e) => panic!("{e:?}"),
                Complete::Error(e) => panic!("{e:?}"),
                Complete::Bytes(_) => unreachable!(),
                Complete::Json(mut v) => {
                    if v.is_empty() {
                        self.end = true
                    } else {
                        if self.reload {
                            self.searched = vec![];
                        }
                        self.searched.append(&mut v);
                        search.next_page();
                    }
                }
            }
        }

        if self.require_new && !self.fetcher.loading() {
            self.fetcher.set_body(&search);
            self.fetcher.send()
        }
    }
    fn move_data_internal(&mut self, ctx: &Context) {
        if self.fetcher.result().is_some() {
            let mut new = SearchRequest::cfetcher_ctx(&get_app_data().url, ctx.clone());
            mem::swap(&mut new, &mut self.fetcher);
            let result = new.take_result().unwrap();
            match result {
                Complete::ApiError(e) => error!("{:?}", e),
                Complete::Error(e) => error!("{:?}", e),
                Complete::Bytes(_) => unreachable!(),
                Complete::Json(mut v) => {
                    if v.is_empty() {
                        self.end = true
                    } else {
                        if self.reload {
                            self.searched = vec![];
                        }
                        self.searched.append(&mut v);
                        get_app_data().search.lock().unwrap().page += 1;
                    }
                }
            }
            self.fetcher
                .set_body(&*get_app_data().search.lock().unwrap());
        }

        if self.require_new && !self.fetcher.loading() {
            self.fetcher.send()
        }
    }
}

fn display_grid<T: DisplaySearch>(
    ui: &mut Ui,
    data: &mut SearchData<T>,
    reset: bool,
    external_only: bool,
    callback: impl FnOnce(String),
) {
    let mut callback = Some(callback);
    let width = match external_only {
        true => 80.,
        false => 200.,
    };
    let height = ui.available_height();
    let itemsxrow = (ui.available_width() / width).floor();
    let size = (ui.available_width() + ui.spacing().item_spacing.x) / itemsxrow - 10.;
    let mut scroll_area = ScrollArea::vertical();
    if reset {
        scroll_area = scroll_area.vertical_scroll_offset(0.0);
    }
    let v = scroll_area
        .drag_to_scroll(true)
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            let app = get_app_data();
            Grid::new("search_grid")
                .num_columns((data.searched.len() as f32 / itemsxrow).ceil() as usize)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    for (index, item) in data.searched.iter().enumerate() {
                        if index != 0 && index % itemsxrow as usize == 0 {
                            ui.end_row();
                        }
                        ui.vertical(|ui| {
                            let image = {
                                if item.internal() {
                                    app.covers.lock().unwrap().get(
                                        item.id_url(),
                                        &item.status(),
                                        &item.ext(),
                                        item.image_number(),
                                        ui.ctx(),
                                    )
                                } else {
                                    app.covers.lock().unwrap().get_url(item.cover(), ui.ctx())
                                }
                            };
                            if let Some(img) = image {
                                let img = img.fit_to_exact_size(vec2(size, size * 1.5));
                                if ui.add(img).clicked() {
                                    if callback.is_some() {
                                        let mut n = None;
                                        mem::swap(&mut n, &mut callback);
                                        if let Some(n) = n {
                                            n(item.id_url().clone());
                                        }
                                    }
                                }
                            } else {
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(size, size * 1.5), Sense::hover());
                                let spinner = Spinner::new();

                                ui.put(rect, spinner);
                            }
                            ui.set_max_width(size);
                            let label = Label::new(get_app_data().get_title(&item.titles()))
                                .wrap()
                                .sense(Sense::click());
                            if ui.add(label).clicked() {
                                if item.internal() {
                                    get_app_data().open(Page::MangaInfo(item.id_url().clone()))
                                } else {
                                    let modifiers = ui.ctx().input(|i| i.modifiers);
                                    ui.ctx().open_url(OpenUrl {
                                        url: item.id_url().to_string(),
                                        new_tab: modifiers.any(),
                                    });
                                }
                            }
                        });
                    }
                });
        });
    data.set_load(|| (v.content_size.y - v.state.offset.y) < (height * 3.));
}

pub struct SearchComponent {
    pub internal: SearchData<SearchResponse>,
    pub external: SearchData<ScrapeSearchResponse>,
    pub external_search: ExternalSearchRequest,
    pub external_change: bool,
    pub reset_scroll: bool,
    pub selected_search: String,
    pub searches: AvailableExternalSitesRequestFetcher,
    pub init: Initter,
}

impl SearchComponent {
    fn init(&mut self, ctx: &Context) {
        if self.init.init() {
            self.internal.fetcher.set_ctx(ctx.clone());
            self.external.fetcher.set_ctx(ctx.clone());
            self.searches.set_ctx(ctx.clone()).send();
        }
    }
    fn get_parser(&mut self, ctx: &Context) -> (Option<Vec<Field>>, bool) {
        self.init(ctx);
        let mut parser = None;
        let mut internal = false;
        if self.selected_search != "internal" {
            self.external
                .move_data_external(ctx, &mut self.external_search);

            if let Some(Complete::Json(v)) = self.searches.result() {
                parser = v.get(&self.selected_search).unwrap().parser();
            }
        } else {
            self.internal.move_data_internal(ctx);
            internal = true;
            parser = Some(vec![Field::new(
                "title".to_string(),
                vec![String::new(), "t".to_string()],
                ItemKind::String,
            )]);
        }
        (parser, internal)
    }

    pub fn search_field_parser<'a>(
        search: &'a mut String,
        allowed: &Vec<Field>,
    ) -> (TextEdit<'a>, Result<Array, String>) {
        let parsed = search_parser(search, false, allowed);
        let color = if parsed.is_err() {
            Some(Color32::from_rgb(255, 64, 64))
        } else {
            None
        };
        let mut search_field = TextEdit::singleline(search);
        if let Some(color) = color {
            search_field = search_field.text_color(color)
        }
        (search_field, parsed)
    }

    pub fn in_panel(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        external_only: bool,
        callback: impl FnOnce(String),
    ) {
        let (parser, internal) = self.get_parser(ctx);
        let search = match internal {
            false => self.external.search.clone(),
            true => "".to_string(),
        };
        let (search_field, parsed) = match parser {
            None => (
                TextEdit::singleline(match internal {
                    true => &mut self.internal.search,
                    false => &mut self.external.search,
                }),
                Ok(Array {
                    not: false,
                    or_post: None,
                    or: false,
                    items: vec![],
                }),
            ),
            Some(ref binding) => Self::search_field_parser(
                match internal {
                    true => &mut self.internal.search,
                    false => &mut self.external.search,
                },
                binding,
            ),
        };
        let (selected, resp) = Self::render_search_bar(
            ui,
            search_field,
            &mut self.selected_search,
            &mut self.searches,
            !external_only,
        );
        self.reset_external(resp, selected, parsed.as_ref().err());

        match internal {
            true => {
                let mut stored = get_app_data().search.lock().unwrap();
                if let Ok(parsed) = parsed {
                    if parsed != stored.query {
                        debug!("{:?}", parsed);
                        stored.query = parsed;
                        stored.page = 1;
                        self.reset_scroll = true;
                        self.internal.reload = true;
                        reset(&mut self.internal.fetcher, stored);
                    }
                }
            }
            false => {
                if search != self.external.search {
                    self.external_search.data.update_query(&search);
                    self.external_change = true;
                }
                if self.external_change {
                    //TODO: duplicate
                    self.external_search.reset_page();
                    self.reset_scroll = true;
                    self.external.reload = true;
                    self.external_change = false;
                    reset_ext(&mut self.external.fetcher, &self.external_search);
                }
            }
        }

        ui.add_space(10.);
        match internal {
            true => display_grid(
                ui,
                &mut self.internal,
                self.reset_scroll,
                external_only,
                callback,
            ),
            false => display_grid(
                ui,
                &mut self.external,
                self.reset_scroll,
                external_only,
                callback,
            ),
        }

        self.reset_scroll = false;
    }

    fn reset_external(&mut self, resp: Response, selected: String, error: Option<&String>) {
        //TODO: set self.external_search.data on change
        // if selected != self.selected_search && self.selected_search != "internal" {
        //     self.external_search.uri.clone_from(&self.selected_search);
        //     if let Some(Complete::Json(v)) = self.searches.result() {
        //         self.external_search.data = match v.get(&self.selected_search).unwrap() {
        //             ValidSearches::String => ExternalSearchData::String(("".to_string(), 1)),
        //             ValidSearches::ValidSearch(_) => ExternalSearchData::Simple(SimpleSearch {
        //                 search: "".to_string(),
        //                 sort: None,
        //                 desc: false,
        //                 status: None,
        //                 tags: vec![],
        //                 page: 1,
        //             }),
        //         };
        //         self.external_change = true;
        //         self.reset_scroll = true;
        //         self.external.reload = true;
        //         reset_ext(&mut self.external.fetcher, &self.external_search);
        //     } else {
        //         unreachable!()
        //     }
        // }
        if let Some(error) = error {
            resp.on_hover_text(error);
        }
    }

    fn render_search_bar(
        ui: &mut Ui,
        search_field: TextEdit,
        selected_search: &mut String,
        searches: &mut AvailableExternalSitesRequestFetcher,
        internal_available: bool,
    ) -> (String, Response) {
        ui.horizontal(|ui| {
            let resp = ui.add(
                search_field
                    .margin(vec2(10., 10.))
                    .hint_text("Advanced Search")
                    .desired_width(ui.available_width() - 140.),
            );
            let selected = selected_search.clone();
            ui.add_enabled_ui(searches.result().is_some(), |ui| {
                let padding = ui.style_mut().spacing.interact_size.y;
                ui.style_mut().spacing.interact_size.y = 33.0;
                ComboBox::new("search_selector", "")
                    .wrap()
                    .selected_text(display_label(selected_search))
                    .show_ui(ui, |ui| {
                        let items = match searches.result() {
                            None => vec![],
                            Some(v) => match v {
                                Complete::Json(v) => v.keys().cloned().collect::<Vec<_>>(),
                                _ => vec!["error".to_string()],
                            },
                        };
                        if internal_available {
                            ui.selectable_value(
                                selected_search,
                                "internal".to_string(),
                                "Internal",
                            );
                        }
                        for item in items {
                            let label = display_label(&item);
                            ui.selectable_value(selected_search, item, label);
                        }
                    });
                ui.style_mut().spacing.interact_size.y = padding;
            });
            (selected, resp)
        })
        .inner
    }
}

impl App for SearchPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Back").clicked() {
                get_app_data().change(Page::Home, vec![Page::Search])
            }
            self.search.in_panel(ui, ctx, false, |id| {
                get_app_data().open(Page::Reader {
                    manga_id: id,
                    chapter_id: None,
                })
            })
        });
    }
}

fn reset(fetcher: &mut SearchRequestFetcher, data: MutexGuard<SearchRequest>) {
    fetcher.set_body(&*data);
    fetcher.send();
}

fn reset_ext(fetcher: &mut ExternalSearchRequestFetcher, data: &ExternalSearchRequest) {
    fetcher.set_body(data);
    fetcher.send();
}

fn display_label(s: &str) -> String {
    let s = s.replace('-', " ");
    if !s.is_empty() {
        s.split(' ')
            .map(|s| format!("{}{}", &s[0..1].to_uppercase(), &s[1..]))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        s
    }
}
