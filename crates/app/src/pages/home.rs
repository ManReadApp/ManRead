use crate::data::image::CoverStorage;
use crate::fetcher::Complete;
use crate::get_app_data;
use crate::requests::{HomeRequestFetcher, RequestImpl};
use crate::widgets::home_page_swithcer::HomePages;
use crate::window_storage::Page;
use api_structure::models::auth::role::Role;
use api_structure::models::manga::search::{Array, Item, ItemData, ItemOrArray, Order};
use api_structure::req::manga::search::SearchRequest;
use api_structure::req::manga::HomeRequest;
use api_structure::resp::manga::home::HomeResponse;
use api_structure::resp::manga::search::SearchResponse;
use eframe::{App, Frame};
use egui::scroll_area::ScrollBarVisibility;
use egui::{vec2, Align, Button, Context, Grid, Label, Layout, ScrollArea, Sense, Spinner, Ui};
use ethread::ThreadHandler;
use poll_promise::Promise;
use std::collections::HashMap;
use std::mem;

pub struct HomePage {
    data: HomeRequestFetcher,
    init: bool,
    moved: bool,
    imgs: Option<ThreadHandler<CoverStorage>>,
}

impl Default for HomePage {
    fn default() -> Self {
        let mut req = HomeRequest::fetcher(&get_app_data().url);
        req.send();
        Self {
            data: req,
            init: false,
            moved: false,
            imgs: None,
        }
    }
}

impl App for HomePage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if !self.init {
            self.init = true;
            self.data.set_ctx(ctx.clone())
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(data) = self.data.result().cloned() {
                if let Complete::Json(v) = data {
                    if self.moved {
                        self.show(ui, &v)
                    } else if let Some(imgs) = &mut self.imgs {
                        if imgs.task.ready().is_some() {
                            let mut promise = Promise::new().1;
                            mem::swap(&mut promise, &mut imgs.task);
                            let covers = promise.block_and_take();
                            {
                                get_app_data().covers.lock().unwrap().append(covers);
                            }
                            self.moved = true;
                            self.show(ui, &v);
                        } else {
                            ui.add(get_app_data().spinner.lock().unwrap().clone().unwrap());
                        }
                    } else {
                        let mut items = vec![];
                        let mut download = |urls: &Vec<SearchResponse>| {
                            items.append(
                                &mut urls
                                    .iter()
                                    .map(|v| {
                                        (v.manga_id.clone(), (v.status, v.ext.clone(), v.number))
                                    })
                                    .collect(),
                            );
                        };
                        download(&v.latest_updates);
                        download(&v.reading);
                        download(&v.random);
                        download(&v.trending);
                        download(&v.newest);
                        download(&v.favorites);
                        let ids = items.into_iter().collect::<HashMap<_, _>>();
                        self.imgs = Some(ThreadHandler::new_async_ctx(
                            CoverStorage::download_many(ids),
                            Some(ui.ctx()),
                        ))
                    }
                } else {
                    data.display_error(ui)
                }
            }
        });
    }
}

impl HomePage {
    fn show(&mut self, ui: &mut Ui, data: &HomeResponse) {
        show_top_bar(ui, HomePages::Home);
        ScrollArea::vertical().show(ui, |ui| {
            show_row(&data.newest, "Newest", ui);
            show_row(&data.trending, "Trending", ui);
            show_row(&data.reading, "Reading", ui);
            show_row(&data.favorites, "Favorites", ui);
            show_row(&data.latest_updates, "Latest Updates", ui);
            show_row(&data.random, "Random", ui);
        });
    }
}

fn show_top_bar(ui: &mut Ui, active: HomePages) {
    let app = get_app_data();
    let role = app.get_user_data().unwrap().role;
    let add_button = matches!(
        role,
        Role::Admin | Role::CoAdmin | Role::Moderator | Role::Author
    );
    ui.horizontal(|ui| {
        for p in HomePages::all() {
            if active == p {
                let _ = ui.selectable_label(true, p.to_string());
            } else if ui.button(p.to_string()).clicked() {
                p.switch_window();
            }
        }

        if add_button && ui.button("Add Manga").clicked() {
            app.open(Page::AddManga)
        }

        if ui.button("Logout").clicked() {
            app.logout()
        }
    });
}

fn show_row(item: &[SearchResponse], label: &str, ui: &mut Ui) {
    let app = get_app_data();
    let v = item
        .iter()
        .map(|v| {
            let id = v.manga_id.clone();
            let title = app.get_title(&v.titles);
            (id, title)
        })
        .collect::<Vec<_>>();
    if !v.is_empty() {
        ui.horizontal(|ui| {
            render_row(label, ui);
        });
        scrollable_items(v, ui, label);
    }
}

fn render_row(label: &str, ui: &mut Ui) {
    ui.label(label);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui.add(Button::new("More").sense(Sense::click())).clicked() {
            let (search, order, desc) = match label {
                "Newest" => (vec![], Order::Created, true),
                "Trending" => {
                    unimplemented!()
                }
                "Reading" => (vec![], Order::LastRead, true),
                "Favorites" => (
                    vec![ItemOrArray::Item(Item {
                        not: false,
                        data: ItemData::enum_("Favorites"),
                    })],
                    Order::Alphabetical,
                    false,
                ),
                "Latest Updates" => (vec![], Order::Updated, true),
                "Random" => (vec![], Order::Random, true),
                _ => unreachable!(),
            };

            *get_app_data().search.lock().unwrap() = SearchRequest {
                order,
                desc,
                limit: 20,
                page: 1,
                query: ItemOrArray::Array(Array {
                    or: false,
                    items: search,
                }),
            };
            get_app_data().open(Page::Search)
        }
    });
}

fn scrollable_items(items: Vec<(String, String)>, ui: &mut Ui, id: &str) {
    let app = get_app_data();
    ScrollArea::horizontal()
        .auto_shrink([true; 2])
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .id_source(id)
        .show(ui, |ui| {
            Grid::new(format!("item_grid_{}", id))
                .num_columns(items.len())
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    for (id, text) in items {
                        ui.vertical(|ui| {
                            let image = { app.covers.lock().unwrap().get_main(&id).cloned() };
                            if let Some(img) = image {
                                let img = img.fit_to_exact_size(vec2(200., 300.));
                                if ui.add(img).clicked() {
                                    get_app_data().open(Page::MangaInfo(id.clone()))
                                }
                            } else {
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(200., 300.), Sense::hover());
                                let spinner = Spinner::new();

                                ui.put(rect, spinner);
                            }

                            let v = ui.allocate_exact_size(vec2(200., 40.), Sense::hover());
                            let label = Label::new(text).sense(Sense::click());
                            if ui.put(v.0, label).clicked() {
                                get_app_data().open(Page::MangaInfo(id))
                            }
                        });
                    }
                });
        });
}
