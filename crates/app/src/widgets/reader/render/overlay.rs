use crate::get_app_data;
use crate::window_storage::Page;
use egui::{pos2, Button, Color32, CursorIcon, Label, Rect, Response, Rounding, Sense, Ui, Widget};

pub struct Overlay {
    name: String,
    episode: f64,
    episode_name: Option<String>,
}

pub struct BottomOverlay {
    visible: bool,
    color: Color32,
}

impl Widget for BottomOverlay {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.add(Button::new("Previous"));
        ui.add(Button::new("Next"));
        ui.add(Button::new("Auto Scroll"));
        ui.add(Button::new("Chapters"));
        ui.add(Button::new("Next"));
        ui.add(Button::new("Settings"))
    }
}

pub struct PageOverlay {
    progress: f64,
    page: i32,
    max_page: i32,
    left: bool,
    percent: bool,
}

impl Overlay {
    pub fn new(name: &str, episode: f64, episode_name: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            episode,
            episode_name: episode_name.map(|v| v.to_string()),
        }
    }
}

impl Widget for Overlay {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), ui.available_size());
        ui.painter().rect_filled(
            rect,
            Rounding::ZERO,
            Color32::from_rgba_unmultiplied(0, 0, 0, 170),
        );
        ui.horizontal_centered(|ui| {
            let grey = Color32::from_rgba_unmultiplied(100, 100, 100, 255);
            let white = Color32::from_rgba_unmultiplied(255, 255, 255, 255);
            ui.style_mut().visuals.override_text_color = Some(white);
            let home = ui.add(Label::new("Home").selectable(false).sense(Sense::click()));
            if home.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if home.clicked() {
                get_app_data().change(
                    Page::Home,
                    vec![Page::Reader {
                        manga_id: "".to_string(),
                        chapter_id: None,
                    }],
                )
            }
            ui.style_mut().visuals.override_text_color = Some(grey);
            ui.add(Label::new(">").selectable(false));
            ui.style_mut().visuals.override_text_color = Some(white);
            let name = ui.add(
                Label::new("One Piece")
                    .selectable(false)
                    .sense(Sense::click()),
            );
            if name.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if name.clicked() {
                get_app_data().change(
                    Page::MangaInfo("".to_string()),
                    vec![Page::Reader {
                        manga_id: "".to_string(),
                        chapter_id: None,
                    }],
                )
            }
            ui.style_mut().visuals.override_text_color = Some(grey);
            ui.add(Label::new(">").selectable(false));
            ui.style_mut().visuals.override_text_color = Some(white);

            ui.add(
                Label::new(match self.episode_name {
                    Some(v) => format!("{}: {}", self.episode, v),
                    None => self.episode.to_string(),
                })
                .selectable(false),
            )
        })
        .response
    }
}
