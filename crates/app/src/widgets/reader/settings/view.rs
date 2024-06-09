#[cfg(target_arch = "wasm32")]
use crate::get_window_dimensions;
use eframe::emath::{vec2, Vec2};
use egui::Context;

pub struct ViewArea {
    pub(crate) margin_top: f32,
    pub(crate) margin_right: f32,
    pub(crate) margin_bottom: f32,
    pub(crate) margin_left: f32,
}

pub fn get_screen_dim(ctx: &Context) -> Vec2 {
    #[cfg(target_arch = "wasm32")]
        let screen = get_window_dimensions();
    #[cfg(not(target_arch = "wasm32"))]
        let screen = ctx.input(|i| i.viewport().outer_rect).unwrap().size();
    screen
}

impl ViewArea {
    pub(crate) fn get_size(&self, ctx: &Context) -> Vec2 {
        get_screen_dim(ctx)
            - vec2(
                self.margin_right + self.margin_left,
                self.margin_top + self.margin_bottom,
            )
    }
}
