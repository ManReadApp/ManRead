use crate::data::user::User;
use crate::fetcher::Complete;
use crate::get_app_data;
use crate::pages::auth::{background, get_background};
use crate::requests::{ActivateRequestFetcher, RequestImpl as _};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::submit_button;
use crate::window_storage::Page;
use api_structure::req::auth::activate::ActivateRequest;
use eframe::{App, Frame};
use egui::{vec2, Context, Image, TextEdit, Ui};
use std::collections::VecDeque;

pub struct VerifyAccountPage {
    height: Option<f32>,
    bg: Image<'static>,
    code: Vec<CodeValue>,
    request: ActivateRequestFetcher,
    init: bool,
}

impl Default for VerifyAccountPage {
    fn default() -> Self {
        Self {
            height: None,
            bg: get_background(),
            code: vec![CodeValue::default(); 6],
            request: ActivateRequest::fetcher(&get_app_data().url),
            init: false,
        }
    }
}

impl App for VerifyAccountPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if !self.init {
            self.init = true;
            self.request.set_ctx(ctx.clone());
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}
impl HoverBackground for VerifyAccountPage {
    fn inner(&mut self, ui: &mut Ui, _: &Context) {
        self.head(ui);
        let focus = text_fields(ui, &mut self.code);
        self.submit(ui, focus);
    }
}

impl VerifyAccountPage {
    fn head(&mut self, ui: &mut Ui) {
        if let Some(v) = self.request.result() {
            v.display_error(ui);
            if let Complete::Json(v) = v {
                let app = get_app_data();
                app.set_user_data(User::new(v.clone()).unwrap());
                User::set_token(v.refresh_token.as_str()).unwrap();
                app.change(Page::Home, Page::all());
            }
        }
        ui.label("Enter the verification code");
        ui.add_space(8.0);
    }

    fn submit(&mut self, ui: &mut Ui, reset_focus: i32) {
        let ec = export_code(&self.code);
        let loading = self.request.loading();
        let btn = submit_button::render(ui, loading, ec.is_some());
        if reset_focus > -1 {
            btn.request_focus();
        }
        if let Some(code) = ec {
            if btn.clicked() {
                self.request.set_body(ActivateRequest { key: code });
                self.request.send();
            }
        }
    }
}

pub fn text_fields(ui: &mut Ui, items: &mut Vec<CodeValue>) -> i32 {
    let pos = textfield_parser(items);
    let mut reset_focus = -1;
    ui.horizontal_top(|ui| {
        let focus = pos.0 + pos.1;
        if focus >= items.len() as i32 {
            reset_focus = pos.0;
        }
        let spaceing = ui.style().spacing.item_spacing.x;
        let len = items.len();
        let space = (ui.available_width() - (40. + spaceing) * len as f32) + spaceing;
        let space = space / (len - 1) as f32;
        let mut count = 0;
        for (index, item) in items.iter_mut().enumerate() {
            let resp = ui.add(
                TextEdit::singleline(&mut item.item)
                    .margin(vec2(0., 10.))
                    .desired_width(40.),
            );
            count += 1;
            if count < len {
                ui.add_space(space);
            }
            if focus > 0 && index == focus as usize {
                resp.request_focus();
            }
            if reset_focus > -1 && index == reset_focus as usize {
                resp.surrender_focus();
            }
        }
    });
    reset_focus
}

fn textfield_parser(items: &mut Vec<CodeValue>) -> (i32, i32) {
    let mut start = -1;
    let mut count = 0;
    let mut chars = VecDeque::new();
    for (index, item) in items.iter_mut().enumerate() {
        let item_value = item.item.replace(['\0', '\n', ' '], "");
        let item_value = match item.value {
            Some(_) => match item_value.is_empty() {
                true => {
                    item.value = None;
                    &item_value[..]
                }
                false => &item_value[1..],
            },
            None => &item_value[..],
        };
        if !item_value.is_empty() {
            chars = item_value.chars().collect::<VecDeque<_>>();
            start = index as i32;
        }
        let front = chars.pop_front();
        if let Some(front) = front {
            item.value = Some(front);
            count += 1;
        }
    }
    for item in items {
        item.item = item.value.unwrap_or_default().to_string();
    }
    (start, count)
}

pub(crate) fn export_code(c: &[CodeValue]) -> Option<String> {
    let mut code = String::new();
    for item in c {
        code.push(match item.value {
            Some(value) => value,
            None => return None,
        });
    }
    Some(code)
}

#[derive(Default, Clone, Debug)]
pub(crate) struct CodeValue {
    value: Option<char>,
    item: String,
}
