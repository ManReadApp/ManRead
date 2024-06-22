use egui::scroll_area::ScrollBarVisibility;
use egui::{Rounding, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};

const BORDER_RADIUS: f32 = 4.0;
const OUTER_MARGIN: f32 = 10.0;
const MAX_WIDTH: f32 = 350.;

//TODO: add drop shadow
pub trait HoverBackground {
    const RENDER_SIDE: bool = false;
    fn inner(&mut self, ui: &mut Ui, ctx: &egui::Context);
    fn side(&mut self, ui: &mut Ui, ctx: &egui::Context) {}
    fn hover_box(&mut self, ui: &mut Ui, ctx: &egui::Context, height: Option<f32>) -> f32 {
        let max_height = ui.available_height();
        let height = height.unwrap_or(max_height).min(max_height);
        let mut out = 0.0;
        let mut sb = StripBuilder::new(ui);
        sb = match Self::RENDER_SIDE {
            true => sb
                .size(Size::remainder())
                .size(Size::relative(0.5).at_most(MAX_WIDTH))
                .size(Size::relative(0.5).at_most(MAX_WIDTH))
                .size(Size::remainder()),
            false => sb
                .size(Size::remainder())
                .size(Size::relative(1.0).at_most(MAX_WIDTH))
                .size(Size::remainder()),
        };
        sb.horizontal(|mut strip| {
            strip.empty();
            let mut content = |main: bool| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::relative(1.).at_most(height))
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.empty();

                            strip.cell(|ui| {
                                ui.painter().rect_filled(
                                    ui.available_rect_before_wrap(),
                                    Rounding::same(BORDER_RADIUS),
                                    ui.style().visuals.panel_fill,
                                );

                                egui::Frame::none()
                                    .outer_margin(OUTER_MARGIN)
                                    .show(ui, |ui| {
                                        ScrollArea::vertical()
                                            .scroll_bar_visibility(
                                                ScrollBarVisibility::AlwaysHidden,
                                            )
                                            .show(ui, |ui| match main {
                                                true => {
                                                    out = ui
                                                        .vertical(|ui| self.inner(ui, ctx))
                                                        .response
                                                        .rect
                                                        .size()
                                                        .y
                                                        + OUTER_MARGIN * 2.0;
                                                }
                                                false => {
                                                    ui.vertical(|ui| self.side(ui, ctx));
                                                }
                                            });
                                    });
                            });
                            strip.empty();
                        });
                })
            };
            content(true);
            if Self::RENDER_SIDE {
                content(false);
            }
            strip.empty()
        });
        if out != height {
            ctx.request_repaint();
        }
        out.min(max_height)
    }
}
