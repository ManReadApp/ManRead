use crate::get_app_data;
use crate::window_storage::Page;

use api_structure::RequestImpl;
use eframe::{App, Frame};
use egui::Context;

pub struct PlaygroundPage {}

impl Default for PlaygroundPage {
    fn default() -> Self {
        Self {}
    }
}

impl App for PlaygroundPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Playground");
            get_app_data().change(
                //Page::Search,
                Page::Reader {
                    manga_id: "3e42gobkidcqyuo6cfyu".to_string(),
                    chapter_id: None,
                },
                Page::all(),
            )
        });
    }
}
