use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use api_structure::models::manga::external_search::ExternalSearchData;
use api_structure::models::manga::tag::Tag;
use api_structure::req::manga::add::AddMangaRequest;
use api_structure::req::manga::external_search::ExternalSearchRequest;
use api_structure::req::manga::search::SearchRequest;
use api_structure::req::manga::{AvailableExternalSitesRequest, KindsRequest};
use eframe::{App, Frame};
use egui::{include_image, vec2, Button, Context, Image, ImageSource, TextBuffer, Ui, Vec2};
use ethread::ThreadHandler;
use rfd::AsyncFileDialog;

use crate::fetcher::{upload_image, UploadFile};
use crate::get_app_data;
use crate::pages::auth::{background, get_background};
use crate::pages::search::{SearchComponent, SearchData};
use crate::requests::{
    AddMangaRequestFetcher, KindsRequestFetcher, RequestImpl as _, SearchRequestFetcher,
};
use crate::widgets::add::{tag_field, Group, SuggestionBox, TagSuggestionBox};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::submit_button;
use crate::window_storage::{Initter, Page};

pub struct AddMangaPage {
    bg: Image<'static>,
    titles: HashMap<String, Vec<String>>,
    title_value: String,
    title_key: String,
    title_key_field: SuggestionBox,
    title_value_field: SuggestionBox,
    kind_suggestion_box: SuggestionBox,
    tag_field: TagSuggestionBox,
    tags: Vec<Tag>,
    sources: Vec<String>,
    kind: String,
    scrape: Option<(String, String)>,
    height: Option<f32>,
    default_image: Image<'static>,
    request: AddMangaRequestFetcher,
    upload_image: Arc<Mutex<Option<Image<'static>>>>,
    uploaded_image: Option<ThreadHandler<Option<String>>>,
    request2: Option<KindsRequestFetcher>,
    init: Initter,
    search: SearchComponent,
}

impl AddMangaPage {
    pub fn new() -> Self {
        let img = Image::new(include_image!("../../assets/upload.png"));
        let request = AddMangaRequest::fetcher(&get_app_data().url);
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
            bg: get_background(),
            titles: HashMap::new(),
            title_value: String::new(),
            title_key: String::new(),
            title_key_field: SuggestionBox::new(
                "title_kind_field",
                [
                    "eng",
                    "jpn",
                    "zh",
                    "ko",
                    "jpn_ascii",
                    "zh_ascii",
                    "ko_ascii",
                    "unknown",
                ]
                .map(|v| v.to_string())
                .to_vec(),
            )
            .default_filter(),
            title_value_field: SuggestionBox::new("title_value_field", vec![]),
            kind_suggestion_box: SuggestionBox::new("kind_field", vec![]).default_filter(),
            tag_field: TagSuggestionBox::new("tag_suggestions"),
            tags: vec![],
            sources: Vec::new(),
            kind: String::new(),
            scrape: None,
            height: None,
            default_image: img,
            request,
            upload_image: Default::default(),
            uploaded_image: None,
            request2: Some(KindsRequest::fetcher(&get_app_data().url)),
            init: Default::default(),
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
                selected_search: "anilist".to_string(),
                searches,
                init: Default::default(),
            },
        }
    }

    fn check(&self) -> bool {
        if self
            .titles
            .values()
            .flatten()
            .collect::<Vec<_>>()
            .is_empty()
        {
            return false;
        }
        if !self.title_value.is_empty() {
            return false;
        }
        if self.kind.is_empty() {
            return false;
        }
        if !self.tag_field.query.is_empty() {
            return false;
        }
        if self
            .scrape
            .as_ref()
            .map(|(a, b)| a.is_empty() || b.is_empty())
            .unwrap_or(false)
        {
            return false;
        }
        if !self.sources.last().unwrap().is_empty() {
            return false;
        }
        if let Some(v) = &self.uploaded_image {
            if let Some(Some(_)) = v.task.ready() {
            } else {
                return false;
            }
        } else {
            return false;
        }
        true
    }

    fn add_manga(&mut self, ui: &mut Ui) {
        if submit_button::render(ui, false, self.check()).clicked() {
            let req = AddMangaRequest {
                names: self.titles.clone(),
                kind: self.kind.clone(),
                tags: self
                    .tags
                    .clone()
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                image_temp_name: self
                    .uploaded_image
                    .take()
                    .unwrap()
                    .task
                    .block_and_take()
                    .unwrap(),
                scape: self.scrape.clone().map(|v| v.0),
                sources: {
                    let mut sources = self.sources.clone()[..self.sources.len() - 1].to_vec();
                    if let Some((_, scrape_str)) = &self.scrape {
                        sources.insert(0, scrape_str.clone());
                    }
                    sources
                },
            };
            self.request.set_body(&req);
            self.request.send();
        }
    }

    fn get_thumb_texture_a_size(&self, ui: &Ui, window_width: f64) -> Image<'static> {
        let calc = |size: Vec2, percentage: f64| {
            let ratio = size.y / size.x;
            let size = window_width * percentage;
            let size = size as f32;
            vec2(size, size * ratio)
        };
        let get_size = |img: &Image<'static>| {
            img.load_for_size(ui.ctx(), ui.available_size())
                .as_ref()
                .ok()
                .and_then(|t| t.size())
                .unwrap_or(Vec2::splat(24.))
        };
        let img_mutex = self.upload_image.lock().unwrap();
        let img = if let Some(v) = &*img_mutex {
            v.clone().fit_to_exact_size(calc(get_size(v), 0.7))
        } else {
            self.default_image
                .clone()
                .fit_to_exact_size(calc(get_size(&self.default_image), 0.15))
        };
        img.clone()
    }
}

impl App for AddMangaPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if self.init.init() {
            if let Some(v) = &mut self.request2 {
                v.set_ctx(ctx.clone()).send();
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}

impl HoverBackground for AddMangaPage {
    fn inner(&mut self, ui: &mut Ui, ctx: &Context) {
        if ui.button("Back").clicked() {
            get_app_data().change(Page::Home, vec![]);
        }
        if let Some(request) = &mut self.request2 {
            if request.result().is_some() {
                if let Some(request) = self.request2.take() {
                    let v = request.take_result().expect("Checked before");
                    match v {
                        crate::fetcher::Complete::ApiError(_) => todo!(),
                        crate::fetcher::Complete::Error(_) => todo!(),
                        crate::fetcher::Complete::Bytes(_) => todo!(),
                        crate::fetcher::Complete::Json(v) => {
                            self.kind_suggestion_box.update_tags(v);
                        }
                    }
                } else {
                    unreachable!()
                }
            }
        }

        if let Some(v) = self.request.result() {
            match v {
                crate::fetcher::Complete::ApiError(err) => {
                    ui.label(err.to_string());
                }
                crate::fetcher::Complete::Error(err) => {
                    ui.label(err.to_string());
                }
                crate::fetcher::Complete::Bytes(_) => {
                    ui.label("Unexpected Bytes as result".to_string());
                }
                crate::fetcher::Complete::Json(v) => {
                    get_app_data().change(Page::MangaInfo(v.to_string()), vec![Page::AddManga])
                }
            }
        }
        if self.sources.is_empty() {
            self.sources.push(String::new());
        }
        let width = ui.available_width();
        let img = self.get_thumb_texture_a_size(ui, width as f64);
        let img = Button::image_and_text(img, "").frame(false);
        ui.vertical_centered(|ui| {
            let img = ui.add(img);
            if img.clicked() {
                let c = ctx;
                let ctx = ctx.clone();
                let up_image = self.upload_image.clone();
                let temp = async move {
                    let file = AsyncFileDialog::new()
                        .add_filter("text", &["png", "jpeg", "jpg", "ico"])
                        .pick_file()
                        .await;
                    if let Some(file) = file {
                        let data = file.read().await;
                        let name = file.file_name();
                        let img = Image::new(ImageSource::Bytes {
                            uri: Cow::Owned(format!("bytes://uploaded_thumb={}", name)),
                            bytes: data.clone().into(),
                        });
                        *up_image.lock().unwrap() = Some(img);
                        let mut data =
                            upload_image(ctx, UploadFile::Bytes(data), Some(name)).await?;
                        if data.is_empty() {
                            return None;
                        }
                        let item = data.remove(0);
                        Some(item.1)
                    } else {
                        None
                    }
                };
                self.uploaded_image = Some(ThreadHandler::new_async_ctx(temp, Some(c)));
            };
        });
        ui.add_space(10.0);

        ui.label("Title:");
        for (key, items) in self.titles.iter_mut() {
            if let Some(_) = Group::new(items).set_title(&format!("{}:", key)).ui(ui) {
                //TODO: move out
            }
        }
        ui.horizontal(|ui| {
            let width = ui.available_width() / 4.0;
            self.title_key_field
                .set_width(width - ui.style().spacing.item_spacing.x);
            self.title_key_field
                .set_popup_width(width * 2.0 - ui.style().spacing.item_spacing.x);
            self.title_value_field
                .set_width(width * 3.0 - ui.style().spacing.item_spacing.x);
            self.title_key_field.show_box(ui, &mut self.title_key);
            if let (_, true) = self.title_value_field.show_box(ui, &mut self.title_value) {
                if !self.title_key.is_empty() && !self.title_value.is_empty() {
                    let value = self.title_value.take();
                    if let Some(v) = self.titles.get_mut(&self.title_key) {
                        v.push(value);
                    } else {
                        self.titles.insert(self.title_key.clone(), vec![value]);
                    }
                }
            }
        });
        ui.label("Tags");
        tag_field(ui, &mut self.tags, &mut self.tag_field);
        ui.label("Kind:");
        self.kind_suggestion_box.show_box(ui, &mut self.kind);
        ui.label("Sources:");
        if self.sources.len() > 1 {
            if let Some(v) = Group::new(&mut self.sources).set_skip(1).ui(ui) {
                let v = self.sources.remove(v);
                if self.sources.last().unwrap().is_empty() {
                    self.sources.remove(self.sources.len() - 1);
                }
                self.sources.push(v);
            }
        }
        if let (_, true) = SuggestionBox::new("url_box".to_lowercase(), vec![])
            .show_box(ui, self.sources.last_mut().unwrap())
        {
            if !self.sources.last().unwrap().is_empty() {
                self.sources.push(String::new());
            }
        }
        let mut scrape = self.scrape.is_some();
        ui.checkbox(&mut scrape, "Scraping");
        if scrape != self.scrape.is_some() {
            match scrape {
                true => self.scrape = Some((String::new(), String::new())),
                false => self.scrape = None,
            };
        }
        if let Some((version, scrape_str)) = &mut self.scrape {
            ui.text_edit_singleline(version);
            ui.text_edit_singleline(scrape_str);
        }

        self.add_manga(ui);
    }

    const RENDER_SIDE: bool = true;

    fn side(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        self.search.in_panel(ui, ctx, true)
    }
}
