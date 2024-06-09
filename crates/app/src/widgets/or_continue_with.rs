use egui::{Color32, InnerResponse, Painter, Pos2, Rect, Stroke};

fn horizontal_line(
    painter: &Painter,
    from: Pos2,
    to: Pos2,
    line_thickness: f32,
    color: Color32,
    fade_out: bool,
) {
    let line_length = to.x - from.x;
    let fade_len = line_length * 0.9;
    if fade_out {
        for i in 0..=fade_len as usize {
            let alpha = ((1.0 - (i as f32 / fade_len).min(1.0)).max(0.) * 255.).round() as u8;
            let fade_color =
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
            let stroke = Stroke::new(line_thickness, fade_color);

            let segment_from = from + (to - from) * (i as f32 / line_length);
            let segment_to = from + (to - from) * ((i + 1) as f32 / line_length);

            painter.line_segment([segment_from, segment_to], stroke);
        }
    } else {
        let empty = line_length - fade_len;
        let from = Pos2::from((from.x + empty, from.y));
        let to = Pos2::from((to.x + empty, to.y));

        for i in 0..=fade_len as usize {
            let alpha = ((1.0 - (i as f32 / fade_len).min(1.0)).max(0.) * 255.).round() as u8;
            let alpha = 255 - alpha;
            let fade_color =
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
            let stroke = Stroke::new(line_thickness, fade_color);

            let segment_from = from + (to - from) * (i as f32 / line_length);
            let segment_to = from + (to - from) * ((i + 1) as f32 / line_length);

            painter.line_segment([segment_from, segment_to], stroke);
        }
    }
}

fn centered_text_widget(ui: &mut egui::Ui, text: &str) -> (Rect, f32, f32) {
    ui.centered_and_justified(|ui| {
        let v = egui::Label::new(text).layout_in_ui(ui).1.rect;
        (ui.add(egui::Label::new(text)).rect, v.min.x, v.max.x)
    })
    .inner
}

pub fn centered_line_with_text_widget(ui: &mut egui::Ui, text: &str) -> InnerResponse<()> {
    ui.horizontal(|ui| {
        let spacing = 5.;
        let v = centered_text_widget(ui, text);
        let line_pos = v.0.min.y + ((v.0.max.y - v.0.min.y) / 2.);
        let min = v.0.min.x;
        let end = v.0.max.x;
        let thickness = 2.;
        let color: Color32 = Color32::from_rgb(100, 100, 100);
        let len = (end - min) / 2.0;
        horizontal_line(
            ui.painter(),
            Pos2::new(min, line_pos),
            Pos2::new(len + min + v.1 - spacing, line_pos),
            thickness,
            color,
            false,
        );
        horizontal_line(
            ui.painter(),
            Pos2::new(len + min + v.2 + spacing, line_pos),
            Pos2::new(end, line_pos),
            thickness,
            color,
            true,
        );
    })
}
