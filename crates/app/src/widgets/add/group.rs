use egui::{
    epaint::{PathStroke, TextShape},
    vec2, Color32, NumExt, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, TextStyle, Ui,
    WidgetText,
};
use std::fmt::Display;

pub struct Group<'a, A: Display> {
    skip: usize,
    data: &'a mut Vec<A>,
    title: Option<&'a str>,
}

impl<'a, A: Display> Group<'a, A> {
    pub fn new(data: &'a mut Vec<A>) -> Self {
        Self {
            skip: 0,
            data,
            title: None,
        }
    }

    pub fn set_skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }
    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl<'a, A: Display> Group<'a, A> {
    pub fn ui(self, ui: &mut Ui) -> Option<usize> {
        ui.horizontal_wrapped(|ui| {
            let mut delete = None;
            let mut clicked = None;
            if let Some(title) = self.title {
                ui.label(title);
            }
            for (index, item) in self.data[..self.data.len() - self.skip].iter().enumerate() {
                let (main, x) = ClosableSelectableLabel::new(item.to_string()).ui(ui);
                if x.clicked() {
                    delete = Some(index);
                }
                if main.double_clicked() {
                    clicked = Some(index);
                }
            }
            if let Some(v) = delete {
                self.data.remove(v);
            }
            clicked
        })
        .inner
    }
}

struct ClosableSelectableLabel {
    text: WidgetText,
}

impl ClosableSelectableLabel {
    fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

impl ClosableSelectableLabel {
    fn ui(self, ui: &mut Ui) -> (Response, Response) {
        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + button_padding;

        let wrap_width = ui.available_width() - total_extra.x;
        let text = self
            .text
            .into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = total_extra + text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        let start = Pos2::new(rect.max.x, rect.min.y);
        let end = Pos2::new(rect.max.x + rect.size().y * 0.45, rect.max.y);
        let response2 = ui.allocate_rect(Rect::from_min_max(start, end), Sense::click());
        if ui.is_rect_visible(response.rect) {
            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                .min;

            let mut visuals = ui.style().interact_selectable(&response, true);

            if response2.is_pointer_button_down_on() || response2.has_focus() {
                visuals = ui.style().visuals.widgets.active;
                visuals.weak_bg_fill = ui.style().visuals.selection.bg_fill;
                visuals.bg_fill = ui.style().visuals.selection.bg_fill;
                visuals.fg_stroke = ui.style().visuals.selection.stroke;
            } else if response2.hovered() || response2.highlighted() {
                visuals = ui.style().visuals.widgets.hovered;
                visuals.weak_bg_fill = ui.style().visuals.selection.bg_fill;
                visuals.bg_fill = ui.style().visuals.selection.bg_fill;
                visuals.fg_stroke = ui.style().visuals.selection.stroke;
            }

            let rect = rect.expand(visuals.expansion);
            let rect = Rect::from_min_max(
                rect.min,
                Pos2::new(rect.max.x + rect.size().y * 0.45 + 4.0, rect.max.y),
            );
            ui.painter().rect(
                rect,
                visuals.rounding,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
            );
            ui.painter().add(
                TextShape::new(text_pos, text, Color32::from_gray(0)).with_underline(Stroke::NONE),
            );
        }

        if ui.is_rect_visible(response2.rect) {
            paint_default_icon(ui.painter(), response2.rect, response2.hovered());
        }

        (response, response2)
    }
}

fn paint_default_icon(painter: &Painter, rect: Rect, hover: bool) {
    let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height() * 0.45));
    let color = match hover {
        true => Color32::from_rgb(255, 0, 0),
        false => Color32::from_gray(220),
    };
    painter.add(Shape::LineSegment {
        points: [rect.right_top(), rect.left_bottom()],
        stroke: PathStroke::new(2.0, color),
    });
    painter.add(Shape::LineSegment {
        points: [rect.left_top(), rect.right_bottom()],
        stroke: PathStroke::new(2.0, color),
    });
}
