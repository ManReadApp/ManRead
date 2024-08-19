use crate::fetcher::Complete;
use crate::requests::MangaInfoRequestFetcher;
use crate::widgets::reader::settings::get_screen_dim;
use crate::widgets::three_dots::ThreeDot;
use crate::window_storage::Page;
use crate::{get_app_data, requests::RequestImpl as _};
use api_structure::req::manga::info::MangaInfoRequest;
use eframe::{App, Frame};
use egui::{
    include_image, pos2, vec2, Align, Context, Image, Layout, Rect, RichText, Sense, Spinner, Ui,
};
use egui_extras::{Column, TableBuilder};

#[allow(dead_code)]
//TODO: implement
pub struct InfoPage {
    info: MangaInfoRequestFetcher,
    fav: Option<bool>,
}

impl InfoPage {
    pub fn new(page: String, ctx: Context) -> Self {
        let mut info = MangaInfoRequest::fetcher(&get_app_data().url);
        info.set_body(MangaInfoRequest { manga_id: page });
        info.set_ctx(ctx);
        info.send();
        Self { info, fav: None }
    }

    fn top_bar(&mut self, ui: &mut Ui, title: &str) {
        let v = get_screen_dim(ui.ctx());
        let size = 20.;
        ui.allocate_ui_at_rect(Rect::from_min_max(pos2(0.0, 0.0), pos2(v.x, size)), |ui| {
            ui.add_space(5.0);
            TableBuilder::new(ui)
                .column(Column::exact(size + 5.))
                .column(Column::exact(v.x - size * 3. - 16. - 10. - 10.))
                .column(Column::exact(size * 2. + 5. + 10.))
                .body(|mut body| {
                    body.row(size, |mut row| {
                        row.col(|ui| {
                            ui.horizontal_top(|ui| {
                                let img = Image::new(match ui.style().visuals.dark_mode {
                                    true => include_image!(
                                        "../assets/icons/back-svgrepo-com-white.svg"
                                    ),
                                    false => include_image!(
                                        "../assets/icons/back-svgrepo-com-black.svg"
                                    ),})
                                .sense(Sense::click());
                                ui.add_space(4.);
                                if ui.add_sized([size -2., size-2.], img).clicked() {
                                    get_app_data().change(Page::Home, vec![Page::MangaInfo(String::default())])
                                }
                            });
                        });
                        row.col(|ui| {
                            let text = RichText::new(title).size(size - 2.).strong();
                            ui.label(text);
                        });
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                ui.add_space(5.);
                                ThreeDot::new("threedot", size - 2., ui.style().visuals.dark_mode).show(
                                    ui,
                                    Actions::None,
                                    |ui, data| {
                                        ui.selectable_value(
                                            data,
                                            Actions::EditManga,
                                            "Edit Manga",
                                        );
                                        ui.selectable_value(
                                            data,
                                            Actions::AddChapter,
                                            "Add Chapter",
                                        );
                                        ui.selectable_value(
                                            data,
                                            Actions::AddToList,
                                            "Add to list",
                                        );
                                        ui.selectable_value(
                                            data,
                                            Actions::ResetProgress,
                                            "Reset progress",
                                        );
                                    },
                                );

                                if let Some(v) = &mut self.fav {
                                    if ui
                                        .add_sized(
                                            [size, size],
                                            Image::new(match *v {
                                                false => match ui.style().visuals.dark_mode {
                                                    true => include_image!(
                                                        "../assets/icons/heart-svgrepo-com-white.svg"
                                                    ),
                                                    false => include_image!(
                                                        "../assets/icons/heart-svgrepo-com-black.svg"
                                                    ),
                                                },
                                                true => include_image!(
                                                "../assets/icons/heart-svgrepo-com-full.svg"
                                            ),
                                            })
                                            .sense(Sense::click()),
                                        )
                                        .clicked()
                                    {
                                        //TODO: send fav
                                        *v = !*v;
                                    }
                                }
                            });
                        });
                    });
                });
        });
    }
}

impl App for InfoPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let copy = unsafe { (self as *mut InfoPage).as_mut().unwrap() };
            if let Some(Complete::Json(v)) = self.info.result() {
                if self.fav.is_none() {
                    self.fav = Some(v.favorite);
                }
                let app = get_app_data();
                let title = app.get_title(&v.titles);
                copy.top_bar(ui, &title);

                ui.label(&v.kind);
                if let Some(v) = &v.description {
                    ui.label(v);
                }
                ui.label(&v.uploader);
                let image = {
                    app.covers.lock().unwrap().get(
                        &v.manga_id,
                        &v.status,
                        &v.cover_ext,
                        v.cover,
                        ctx,
                    )
                };
                if let Some(img) = image {
                    let img = img.fit_to_exact_size(vec2(200., 300.));
                    ui.add(img);
                } else {
                    let (rect, _) = ui.allocate_exact_size(vec2(200., 300.), Sense::hover());
                    let spinner = Spinner::new();

                    ui.put(rect, spinner);
                }
                if match &v.progress {
                    Some(_) => ui.button("Continue Reading"),
                    None => ui.button("Start Reading"),
                }
                .clicked()
                {
                    app.change(
                        Page::Reader {
                            manga_id: v.manga_id.clone(),
                            chapter_id: None,
                        },
                        vec![Page::MangaInfo(String::new())],
                    );
                }
                if !v.tags.is_empty() {
                    buttons(v.tags.iter().map(|v| v.to_string()).collect(), ui);
                }
                if !v.authors.is_empty() {
                    buttons(v.authors.clone(), ui);
                }
                if !v.artists.is_empty() {
                    buttons(v.artists.clone(), ui);
                }
                //title bar: favorite,visibility,my
                //custom_grid: chapters,v.progress
                //icon: sources,scraper
                //duno: relations,
            } else {
                self.top_bar(ui, "Loading...");
                ui.spinner();
            }
        });
    }
}

fn buttons(items: Vec<String>, ui: &mut Ui) {
    //TODO: color
    ui.horizontal_wrapped(|ui| {
        for label in items {
            let _ = ui.selectable_label(true, label);
        }
    });
}

#[derive(PartialEq)]
//TODO: implement
#[allow(dead_code)]
enum Actions {
    None,
    AddChapter,
    EditManga,
    AddToList,
    ResetProgress,
}

// impl InfoPage {
//     fn update2(&mut self, ctx: &Context, frame: &mut Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| match self.data.get(ctx) {
//             Status::Loaded(v) => {
//                 #[cfg(target_arch = "wasm32")]
//                     let screen = get_window_dimensions();
//                 #[cfg(not(target_arch = "wasm32"))]
//                     let screen = frame.info().window_info.size;
//                 const BANNER_HEIGHT: f32 = 200.;
//                 match self.cover.get(ctx) {
//                     Status::Loaded(cover) => {
//                         egui::CentralPanel::default().show_inside(ui, |ui| {
//                             let hide_banner = if let Some(pos) = self.thumb_shadow_pos {
//                                 pos <= 0.0
//                             } else {
//                                 false
//                             };
//                             if !hide_banner {
//                                 let img = self.thumb.get_or_insert_with(|| {
//                                     let bytes = cover.image.lock().unwrap().get_image_bytes();
//                                     let mut start = cover.height() as f32 * 0.25;
//                                     let mut height =
//                                         cover.width() as f32 / screen.x * BANNER_HEIGHT;
//                                     let h_over = (height - cover.height() as f32 + start).max(0.0);
//                                     start -= h_over;
//                                     if height > cover.height() as f32 {
//                                         start = 0.0;
//                                         height = cover.height() as f32;
//                                     }
//
//                                     let img = image::load_from_memory(&bytes).unwrap().crop(
//                                         0,
//                                         start as u32,
//                                         cover.width() as u32,
//                                         height as u32,
//                                     );
//                                     let img = img.blur(4.0);
//                                     MyImage::from_image("thumb", img)
//                                 });
//                                 img.put_image(
//                                     ui,
//                                     pos2(0.0, 0.0),
//                                     vec2(screen.x, BANNER_HEIGHT),
//                                     None,
//                                 );
//                             }
//
//                             if let Some(pos) = self.thumb_shadow_pos {
//                                 let img = self.thumb_shadow.get_or_insert_with(|| {
//                                     let bytes = cover.image.lock().unwrap().get_image_bytes();
//                                     let mut image = image::load_from_memory(&bytes).unwrap();
//                                     let image = get_image(&mut image);
//                                     MyImage::from_image("thumb_shadow", image)
//                                 });
//                                 img.put_image(
//                                     ui,
//                                     pos2(0.0, pos),
//                                     vec2(screen.x, BANNER_HEIGHT),
//                                     None,
//                                 );
//                                 let size = cover.size_vec2();
//                                 let desired_height = BANNER_HEIGHT * 1.25;
//                                 let desired_width_mult = 0.25;
//                                 let mut size =
//                                     vec2(desired_height / size.y * size.x, desired_height);
//                                 if size.x > screen.x * desired_width_mult {
//                                     size = vec2(
//                                         screen.x * desired_width_mult,
//                                         screen.x * desired_width_mult / size.x * size.y,
//                                     );
//                                 }
//                                 let rect =
//                                     Rect::from_min_size(pos2(5., pos - BANNER_HEIGHT + 5.), size);
//
//                                 //TODO: skip painting when not visible
//                                 cover.put_image(ui, rect.min, rect.size(), None);
//                                 if v.status.to_i32() != 0 {
//                                     let painter = ui.painter_at(rect);
//                                     let galley = painter.layout_no_wrap(
//                                         v.status.to_string().to_ascii_uppercase(),
//                                         FontId::monospace(9.0),
//                                         Color32::WHITE,
//                                     );
//
//                                     //FIX: The correction is to get some padding, the solution isnt optimal because its a fixed number
//                                     const CORRECTION: f32 = 5.0;
//
//                                     let start = (galley.size().x.powi(2) / 2.).sqrt() + CORRECTION;
//                                     let thickness = galley.size().y / (PI / 6.0).sin();
//                                     let color = v.status.color();
//                                     let shape = PathShape {
//                                         points: vec![
//                                             pos2(start, 0.0),
//                                             pos2(start + thickness, 0.0),
//                                             pos2(0.0, start + thickness),
//                                             pos2(0.0, start),
//                                         ],
//                                         closed: true,
//                                         fill: Color32::from_rgb(color[0], color[1], color[2]),
//                                         stroke: Stroke::new(0.0, Color32::from_rgb(0, 0, 0)),
//                                     };
//                                     painter.add(shape);
//
//                                     let shape = TextShape {
//                                         pos: pos2(CORRECTION, start - CORRECTION * 0.25),
//                                         galley,
//                                         underline: Default::default(),
//                                         override_text_color: None,
//                                         angle: -PI / 4.0,
//                                     };
//                                     painter.add(shape);
//                                 }
//                             }
//                             // show everything but ongoing, used i32 bc Status is already imported
//                         });
//                     }
//                     Status::Waiting => {
//                         ui.add(Spinner::default());
//                     }
//                     Status::Error(_) => {
//                         ui.label("Error");
//                     }
//                     Status::None => {
//                         ui.label("None");
//                         self.cover.retry(ctx);
//                     }
//                 }
//                 ScrollArea::vertical()
//                     .auto_shrink([true; 2])
//                     .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
//                     .id_source("scroll_test")
//                     .show(ui, |ui| {
//                         let vv = ui.allocate_exact_size(
//                             vec2(0.0, BANNER_HEIGHT - 2. * 5.),
//                             Sense::hover(),
//                         );
//                         ui.add_space(5.0);
//                         ui.horizontal(|ui| {
//                             ui.group(|ui| {
//                                 //TODO:
//                                 ui.selectable_label(true, "Chapters");
//                                 ui.button("Info");
//                                 ui.button("Art");
//                                 ui.button("Related");
//                             });
//                         });
//
//                         self.thumb_shadow_pos = Some(vv.0.max.y + 1.0);
//                         let mut titles = v.titles.clone();
//                         let title = get_title(&vec!["eng".into()], &titles);
//                         if let Some((ident, _)) = &title {
//                             titles.get_mut(ident).unwrap().remove(0);
//                         }
//                         let title = title.map(|(_, v)| v).unwrap_or("No title found".into());
//                         let other_titles = titles.into_values().flatten().collect::<Vec<_>>();
//                         ui.heading(format!("Title: {}", title));
//                         ui.label(format!("Other titles: {}", other_titles.join(", ")));
//                         if let Some(disc) = &v.description {
//                             ui.label(format!("Discription: {}", disc));
//                         }
//                         let tags = &v.tags;
//                         ui.label(format!(
//                             "Tags: {}",
//                             tags.iter()
//                                 .map(|v| v.to_string())
//                                 .collect::<Vec<_>>()
//                                 .join(", ")
//                         ));
//                         let author = &v.uploader;
//                         ui.label(format!("Author: {}", author));
//                         let kind = &v.kind;
//                         ui.label(format!("Kind: {}", kind));
//                         let sources = &v.sources;
//                         ui.label(format!(
//                             "Sources: {}",
//                             sources
//                                 .iter()
//                                 .map(|v| v.to_string())
//                                 .collect::<Vec<_>>()
//                                 .join(", ")
//                         ));
//                         let relations = &v.relations;
//                         ui.label(format!(
//                             "Relations: {}",
//                             relations
//                                 .iter()
//                                 .map(|v| v.to_string())
//                                 .collect::<Vec<_>>()
//                                 .join(", ")
//                         ));
//
//                         let fav = v.favorite;
//                         ui.label(format!("Fav: {}", fav));
//                         let owner = v.owner;
//                         ui.label(format!("Owner: {}", owner));
//                         let mut chapters = vec![];
//                         for _ in 0..200 {
//                             chapters.append(&mut v.chapters.clone())
//                         }
//                         let co = 5;
//                         TableBuilder::new(ui)
//                             .vscroll(false)
//                             .columns(Column::remainder(), co)
//                             .body(|mut body| {
//                                 for ch in chapters.chunks(co) {
//                                     body.row(30.0, |mut row| {
//                                         for ch in ch {
//                                             row.col(|ui| {
//                                                 ui.label(format!("{}", ch.chapter));
//                                             });
//                                         }
//                                     });
//                                 }
//                             });
//                     });
//             }
//             Status::Waiting => {
//                 ui.add(Spinner::default());
//             }
//             Status::Error(_) => {
//                 ui.label("Error");
//             }
//             Status::None => {
//                 ui.label("None");
//                 self.data.retry(ctx);
//             }
//         });
//     }
// }
//
// fn get_image(image: &mut DynamicImage) -> DynamicImage {
//     // Load the base image and create a copy to overlay the gradient on
//     let output_image = image.crop(
//         0,
//         (image.height() as f32 * 0.35).round() as u32,
//         image.width(),
//         (image.height() as f32 * 0.35).round() as u32,
//     );
//     let mut output_image = output_image
//         .thumbnail(
//             256,
//             (256.0 / output_image.width() as f32 * output_image.height() as f32) as u32,
//         )
//         .to_rgba8();
//
//     // Define gradient colors
//     //TODO: dark mode
//     let end_color_dark = ImageRgba([26, 26, 28, 255]); //#191a1c
//     //let end_color_bright = ImageRgba([255, 255, 255, 255]); //#ffffff
//     let start_color_dark = ImageRgba([26, 26, 28, 204]); //rgba(26,26,28,0.8)
//     // let start_color_bright = ImageRgba([255, 255, 255, 204]); //rgba(255,255,255,0.8)
//
//     // Create the gradient image
//     let gradient_image = create_half_circle_gradient(
//         output_image.width(),
//         output_image.height(),
//         start_color_dark,
//         end_color_dark,
//         output_image.height() as f32 * 0.85,
//     );
//
//     // Overlay the gradient on the output image
//     output_image = image_overlay(&output_image, &gradient_image);
//     let image = DynamicImage::ImageRgba8(output_image);
//     image.blur(12.0)
// }
//
// fn create_half_circle_gradient(
//     width: u32,
//     height: u32,
//     start_color: ImageRgba<u8>,
//     end_color: ImageRgba<u8>,
//     radius: f32,
// ) -> RgbaImage {
//     let mut gradient_image = RgbaImage::new(width, height);
//
//     let center_x = width as f32 / 2.0;
//     let center_y = 0.0; // Start from the top
//
//     for y in 0..height {
//         for x in 0..width {
//             let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
//             let ratio = (distance / radius).min(1.0).max(0.0);
//             let interpolated_color = interpolate_color(start_color, end_color, ratio);
//             gradient_image.put_pixel(x, y, interpolated_color);
//         }
//     }
//     gradient_image
// }
