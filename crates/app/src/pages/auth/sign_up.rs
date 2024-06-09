use crate::fetcher::upload_image;
use crate::get_app_data;
use crate::pages::auth::{background, get_background};
use crate::util::validator::{validate_email, validate_password, validate_username};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::submit_button;
use crate::window_storage::Page;
use eframe::{App, Frame};
use egui::{vec2, Context, Image, Label, Link, TextEdit, Ui};
use ethread::ThreadHandler;
use identicon_rs::Identicon;
use image::ImageFormat;
use std::io::Cursor;

pub struct SignUpPage {
    height: Option<f32>,
    bg: Image<'static>,
    pub username: String,
    pub email1: String,
    email2: String,
    pub password1: String,
    password2: String,
    pub thumb: Option<ThreadHandler<(Option<Vec<(String, String)>>, Image<'static>)>>,
}

impl Default for SignUpPage {
    fn default() -> Self {
        Self {
            height: None,
            bg: get_background(),
            username: "".to_string(),
            email1: "".to_string(),
            email2: "".to_string(),
            password1: "".to_string(),
            password2: "".to_string(),
            thumb: None,
        }
    }
}

impl App for SignUpPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        self.change(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}
impl HoverBackground for SignUpPage {
    fn inner(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.vertical_centered(|ui| {
            self.header(ui);
            self.fields(ui);
            self.submit_button(ui, ctx);
            self.already_a_member(ui);
        });
    }
}

impl SignUpPage {
    fn change(&self, ctx: &Context) {
        if let Some(v) = &self.thumb {
            if v.task.ready().is_some() {
                get_app_data().open(Page::SignUpInfo);
                ctx.request_repaint();
            }
        }
    }
    fn text_field(s: &mut String, label: &str, password: bool, ui: &mut Ui) {
        ui.add(
            TextEdit::singleline(s)
                .hint_text(label)
                .margin(vec2(0., 20.))
                .desired_width(f32::INFINITY)
                .password(password),
        );
    }
    fn header(&mut self, ui: &mut Ui) {
        ui.heading("You're new here?");
        ui.add_space(4.0);
        ui.add(Label::new("Sign up here"));
        ui.add_space(4.0);
    }
    fn fields(&mut self, ui: &mut Ui) {
        Self::text_field(&mut self.username, "Username", false, ui);
        ui.add_space(4.0);
        Self::text_field(&mut self.email1, "Email", false, ui);
        ui.add_space(4.0);
        Self::text_field(&mut self.email2, "Email again", false, ui);
        ui.add_space(4.0);
        Self::text_field(&mut self.password1, "Password", true, ui);
        ui.add_space(4.0);
        Self::text_field(&mut self.password2, "Password again", true, ui);
    }

    fn submit_button(&mut self, ui: &mut Ui, ctx: &Context) {
        let email =
            self.email1 == self.email2 && self.email1.len() > 4 && validate_email(&self.email1);
        let password = self.password1 == self.password2 && validate_password(&self.password1);
        let username = self.username.len() >= 3 && validate_username(&self.username); //TODO: db check
        ui.add_space(8.0);
        let loading = false;
        let btn = submit_button::render(ui, loading, email && password && username && !loading);
        if btn.clicked() {
            let username = self.username.clone();
            self.thumb = Some(ThreadHandler::new_async_ctx(
                upload(username, ctx.clone()),
                Some(ctx),
            ));
        }
        ui.add_space(8.0);
    }

    fn already_a_member(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.label("Already a member?");
            if ui
                .add(Link::new("Login here"))
                .on_hover_text("Click to login")
                .clicked()
            {
                get_app_data().open(Page::SignIn);
            }
        });
    }
}

async fn upload(username: String, ctx: Context) -> (Option<Vec<(String, String)>>, Image<'static>) {
    let identicon_conways_glider = Identicon::new(&username);
    let img = identicon_conways_glider.generate_image().unwrap();
    let mut res = Cursor::new(vec![]);
    img.write_to(&mut res, ImageFormat::Png).unwrap();
    let data = upload_image(ctx, res.get_ref().clone().into(), None).await;
    let img = Image::from_bytes("generated_icon", res.into_inner());
    (data, img)
}
