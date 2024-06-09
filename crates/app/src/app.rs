use crate::fonts::setup_custom_fonts;
use crate::get_app_data;
use crate::window_storage::Windows;
use eframe::Frame;
use egui::Context;

#[derive(Default)]
pub struct TemplateApp {
    windows: Windows,
    init: bool,
}

fn init(ctx: &Context) {
    egui_extras::install_image_loaders(ctx);
    setup_custom_fonts(ctx.clone());
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if !self.init {
            self.init = true;
            init(ctx);
        }
        // cleanup old
        if let Some(pages) = get_app_data().change_window() {
            self.windows.dispose_many(pages);
        }
        // get page
        let page = get_app_data().page();
        // render page
        self.windows.get_app(page).update(ctx, frame);
    }
}
