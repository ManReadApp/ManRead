use eframe::emath::{vec2, Pos2, Rect};
use egui::{Image, Ui};
use std::ops::Add;

pub mod reset_password;
pub mod sign_in;
pub mod sign_up;
pub mod sign_up_info;
pub mod verify_account;

pub fn background(image: &Image, ui: &mut Ui) {
    let size = ui.style().spacing.item_spacing.x * 2.0;
    let rect = Rect::from_min_max(
        Pos2::ZERO,
        ui.available_size().add(vec2(size, size + 3.0)).to_pos2(),
    );
    image.paint_at(ui, rect);
}

pub fn get_background() -> Image<'static> {
    Image::new(egui::include_image!("../../assets/background.png"))
}
