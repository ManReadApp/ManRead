use eframe::emath::{pos2, vec2, Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontId, PathShape, TextShape};
use egui::epaint::PathStroke;
use egui::{Image, Response, Ui, Widget};
use std::f64::consts::PI;
use std::ops::Add;

#[derive(Clone)]
pub struct ImageOverlay {
    image: Image<'static>,
    text: Option<String>,
    fontsize: f32,
    banner_color: [u8; 3],
    dot: Dot,
    dot_size: f32,
    flag: Option<Image<'static>>,
    fixed_banner_len: Option<f32>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum Dot {
    None,
    Red,
    Green,
    Blue,
    Yellow,
    Grey,
    Purple,
}

impl Dot {
    fn color(self) -> Color32 {
        match self {
            Dot::None => unreachable!(),
            Dot::Red => Color32::from_rgb(255, 64, 64),
            Dot::Green => Color32::from_rgb(4, 208, 0),
            Dot::Blue => Color32::from_rgb(0, 201, 245),
            Dot::Yellow => Color32::from_rgb(218, 117, 0),
            Dot::Grey => Color32::from_rgb(196, 196, 196),
            Dot::Purple => Color32::from_rgb(125, 64, 255),
        }
    }
}

impl ImageOverlay {
    pub fn ongoing(image: Image<'static>) -> Self {
        Self {
            image,
            text: None,
            fontsize: 0.0,
            banner_color: [0; 3],
            dot: Dot::Blue,
            dot_size: 5.0,
            flag: None,
            fixed_banner_len: None,
        }
    }

    pub fn hiatus(image: Image<'static>) -> Self {
        Self {
            image,
            text: None,
            fontsize: 0.0,
            banner_color: [0; 3],
            dot: Dot::Grey,
            dot_size: 5.0,
            flag: None,
            fixed_banner_len: None,
        }
    }
    pub fn upcoming(image: Image<'static>) -> Self {
        Self {
            image,
            text: None,
            fontsize: 0.0,
            banner_color: [0; 3],
            dot: Dot::Purple,
            dot_size: 5.0,
            flag: None,
            fixed_banner_len: None,
        }
    }
    pub fn dropped(image: Image<'static>) -> Self {
        Self {
            image,
            text: Some("DROPPED".to_string()),
            fontsize: 10.0,
            banner_color: [255, 64, 64],
            dot: Dot::None,
            dot_size: 5.0,
            flag: None,
            fixed_banner_len: Some(60.),
        }
    }
    pub fn completed(image: Image<'static>) -> Self {
        Self {
            image,
            text: Some("COMPLETED".to_string()),
            fontsize: 10.0,
            banner_color: [4, 208, 0],
            dot: Dot::None,
            dot_size: 5.0,
            flag: None,
            fixed_banner_len: Some(60.),
        }
    }

    pub fn fit_to_exact_size(mut self, size: Vec2) -> Self {
        self.image = self.image.fit_to_exact_size(size);
        self
    }
}
impl Widget for ImageOverlay {
    fn ui(self, ui: &mut Ui) -> Response {
        let img = ui.add(self.image);
        let min = img.rect.min.to_vec2();
        let painter = ui.painter();
        if let Some(text) = self.text {
            let galley =
                painter.layout_no_wrap(text, FontId::monospace(self.fontsize), Color32::WHITE);
            let text_size = galley.size();
            let banner_size = self.fixed_banner_len.unwrap_or(text_size.x);
            let (first, second) = get_info(banner_size, self.fontsize + 1.);
            let banner = PathShape {
                points: get_points(min, first, second),
                closed: true,
                fill: Color32::from_rgb(
                    self.banner_color[0],
                    self.banner_color[1],
                    self.banner_color[2],
                ),
                stroke: PathStroke::new(0.0, Color32::from_rgb(0, 0, 0)),
            };
            painter.add(banner);
            let mov = match self.fixed_banner_len {
                None => Vec2::ZERO,
                Some(fixed_banner_len) => {
                    let size = fixed_banner_len - text_size.x;
                    if size < 0.0 {
                        Vec2::ZERO
                    } else {
                        vec2(first, -first) * (size * 0.5 / fixed_banner_len)
                    }
                }
            };
            let text = TextShape {
                pos: pos2(0.0, first).add(min).add(mov),
                galley,
                underline: Default::default(),
                fallback_color: Default::default(),
                override_text_color: None,
                opacity_factor: 1.0,
                angle: -ROTATION as f32,
            };
            painter.add(text);
        }
        if !matches!(self.dot, Dot::None) {
            let pos =
                pos2(img.rect.max.x, min.y) - vec2(self.dot_size, -self.dot_size) - vec2(3., -3.);

            painter.circle_filled(pos, self.dot_size, self.dot.color());
        }
        if let Some(flag) = self.flag {
            let size = vec2(20., 20.);
            let flag = flag.fit_to_exact_size(size);
            let rect = Rect::from_min_size(img.rect.max - size - vec2(3., 1.), size);
            ui.put(rect, flag);
        }

        img
    }
}

const ROTATION: f64 = 45. * PI / 180.;

/// point 1 = (first, 0)
/// point 2 = (first+second, 0)
/// point 3 = (0, first+second)
/// point 4 = (0, first)
/// point 5 = (first, 0)
/// text rotation point = (text_height_px, first+second)
/// text rotation angle = 45 degree = ROTATION rad
fn get_info(text_len_px: f32, text_height_px: f32) -> (f32, f32) {
    // a^2+a^2 = text_len_px^2  => 2a^2 == text_len_px^2
    let first = (text_len_px.powi(2) / 2.0).sqrt();
    //sin(a) =gk/hy => hy = gk/sin(a)
    let sina: f64 = ROTATION.sin();
    let second = text_height_px as f64 / sina;
    (first, second as f32)
}

fn get_points(min: Vec2, first: f32, second: f32) -> Vec<Pos2> {
    vec![
        pos2(first, 0.0),
        pos2(first + second, 0.0),
        pos2(0.0, first + second),
        pos2(0.0, first),
    ]
    .into_iter()
    .map(|pos| pos.add(min))
    .collect()
}
