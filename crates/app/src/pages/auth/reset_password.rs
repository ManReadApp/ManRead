use crate::data::user::User;
use crate::fetcher::{Complete, Fetcher};
use crate::get_app_data;
use crate::pages::auth::verify_account::{text_fields, CodeValue};
use crate::pages::auth::{background, get_background};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::submit_button;
use crate::window_storage::Page;
use api_structure::auth::jwt::JWTs;
use api_structure::auth::reset_password::RequestResetPasswordRequest;
use api_structure::auth::reset_password::ResetPasswordRequest;
use api_structure::RequestImpl;
use eframe::{App, Frame};
use egui::{vec2, Context, Image, Label, Link, Sense, TextEdit, Ui};

pub struct ResetPasswordPage {
    height: Option<f32>,
    bg: Image<'static>,
    username: String,
    sent: bool,
    email_reset: bool,
    password: String,
    request: Fetcher<()>,
    reset: Fetcher<JWTs>,
    init: bool,
    code: Vec<CodeValue>,
}

impl Default for ResetPasswordPage {
    fn default() -> Self {
        Self {
            height: None,
            bg: get_background(),
            username: "".to_string(),
            sent: false,
            email_reset: false,
            password: "".to_string(),
            request: Fetcher::new(
                RequestResetPasswordRequest::request(&get_app_data().url).unwrap(),
            ),
            reset: Fetcher::new(ResetPasswordRequest::request(&get_app_data().url).unwrap()),
            init: false,
            code: vec![CodeValue::default(); 6],
        }
    }
}

impl App for ResetPasswordPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if !self.init {
            self.init = true;
            self.request.set_ctx(ctx.clone());
            self.reset.set_ctx(ctx.clone());
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}
impl HoverBackground for ResetPasswordPage {
    fn inner(&mut self, ui: &mut Ui, _: &Context) {
        ui.vertical_centered(|ui| {
            if let Some(v) = self.reset.result() {
                v.display_error(ui);
                if let Complete::Json(json) = v {
                    let app = get_app_data();
                    app.set_user_data(User::new(json.clone()).unwrap());
                    User::set_token(json.refresh_token.as_str()).unwrap();
                    app.change(Page::Home, Page::all())
                }
            }
            self.header(ui);
            self.body(ui);
            self.go_back(ui);
        });
    }
}

impl ResetPasswordPage {
    fn header(&mut self, ui: &mut Ui) {
        ui.heading("Forgot password?");
        ui.add_space(4.0);
        if ui
            .add(
                Label::new(
                    "Enter your email address and we'll send you a link to reset your password.",
                )
                .sense(Sense::click()),
            )
            .on_hover_text("Click to switch to email reset")
            .clicked()
        {
            self.email_reset = !self.email_reset;
        }
        ui.add_space(8.0);
    }

    fn submit(&mut self, reset_focus: i32, ui: &mut Ui) {
        let ec = crate::pages::auth::verify_account::export_code(&self.code);
        let loading = self.reset.loading();
        let btn = submit_button::render(ui, loading, ec.is_some());
        if reset_focus > -1 {
            btn.request_focus();
        }
        if let Some(code) = ec {
            if btn.clicked() {
                self.reset.set_body(ResetPasswordRequest {
                    ident: self.username.clone(),
                    email: self.email_reset,
                    key: code,
                    password: self.password.clone(),
                });
                self.reset.send();
            }
        }
    }

    fn body(&mut self, ui: &mut Ui) {
        if self.sent {
            ui.label("Email sent!");
            ui.add_space(4.0);
            ui.add(
                TextEdit::singleline(&mut self.password)
                    .hint_text("Password")
                    .margin(vec2(0., 20.))
                    .password(true)
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(4.0);
            let focus = text_fields(ui, &mut self.code);
            ui.add_space(4.0);
            self.submit(focus, ui)
        } else {
            ui.add(
                TextEdit::singleline(&mut self.username)
                    .hint_text(match self.email_reset {
                        true => "Email",
                        false => "Username",
                    })
                    .margin(vec2(0., 20.))
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(8.0);
            if ui.button("Reset Password").clicked() {
                self.sent = true;
                self.request.set_body(RequestResetPasswordRequest {
                    ident: self.username.clone(),
                    email: self.email_reset,
                });
                self.request.send()
            }
            ui.add_space(4.0);
        }
    }

    fn go_back(&self, ui: &mut Ui) {
        let page = get_app_data().go_back_page.lock().unwrap().take();
        let mut link_text = "Login here";
        let mut hover = "Click to login";
        if page.is_some() {
            link_text = "Go back";
            hover = "Click to go back";
        }
        if ui.add(Link::new(link_text)).on_hover_text(hover).clicked() {
            get_app_data().change(page.unwrap_or(Page::SignIn), vec![Page::ResetPassword]);
        }
    }
}
