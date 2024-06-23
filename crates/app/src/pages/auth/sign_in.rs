use crate::data::user::User;
use crate::fetcher::Complete;
use crate::get_app_data;
use crate::pages::auth::{background, get_background};
use crate::requests::{LoginRequestFetcher, RequestImpl as _};
use crate::util::validator::{validate_email, validate_password, validate_username};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::or_continue_with::centered_line_with_text_widget;
use crate::widgets::submit_button;
use crate::window_storage::Page;
use api_structure::models::auth::login::{LoginWithEmailAndPassword, LoginWithUsernameAndPassword};
use api_structure::models::auth::role::Role;
use api_structure::req::auth::login::LoginRequest;
use eframe::{App, Frame};
use egui::{
    vec2, Align, Context, CursorIcon, Image, ImageButton, Label, Layout, Link, Sense, TextEdit, Ui,
};
pub struct LoginPage {
    height: Option<f32>,
    bg: Image<'static>,
    gh: ImageButton<'static>,
    apple: ImageButton<'static>,
    google: ImageButton<'static>,
    email_login: bool,
    username: String,
    password: String,
    request: LoginRequestFetcher,
    ctx_set: bool,
}
impl Default for LoginPage {
    fn default() -> Self {
        //TODO: icons look bad
        //egui_extras::image::gif_to_sources(concat!("gif://", ""), Cursor::new(include_bytes!("../../assets/loading.gif")));
        Self {
            height: None,
            bg: get_background(),
            gh: ImageButton::new(
                Image::new(egui::include_image!("../../assets/logos/github.png"))
                    .fit_to_exact_size(vec2(32., 32.)),
            ),
            apple: ImageButton::new(
                Image::new(egui::include_image!("../../assets/logos/apple.png"))
                    .fit_to_exact_size(vec2(32., 32.)),
            ),
            google: ImageButton::new(
                Image::new(egui::include_image!("../../assets/logos/google.png"))
                    .fit_to_exact_size(vec2(32., 32.)),
            ),
            email_login: false,
            username: "".to_string(),
            password: "".to_string(),
            request: LoginRequest::fetcher(&get_app_data().url),
            ctx_set: false,
        }
    }
}

impl App for LoginPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}

impl HoverBackground for LoginPage {
    fn inner(&mut self, ui: &mut Ui, ctx: &Context) {
        if !self.ctx_set {
            self.ctx_set = true;
            self.request.set_ctx(ctx.clone());
        }
        ui.vertical_centered(|ui| {
            self.header(ui);
            self.username_field(ui);
            self.password_field(ui);
            self.forgot_password(ui);
            self.submit_button(ui);
            self.or_continue_with(ui);
            self.not_a_member(ui);
        });
    }
}

impl LoginPage {
    fn header(&mut self, ui: &mut Ui) {
        if let Some(v) = self.request.result() {
            v.display_error(ui);
            if let Complete::Json(json) = v {
                let app = get_app_data();
                app.set_user_data(User::new(json.clone()).unwrap());
                User::set_token(json.refresh_token.as_str()).unwrap();
                if Role::NotVerified == app.get_user_data().unwrap().role {
                    app.open(Page::VerifyAccount)
                } else {
                    app.change(Page::Home, Page::all())
                }
            }
        }
        ui.heading("Welcome back");
        ui.add_space(4.0);
        if ui
            .add(Label::new("We've missed you").sense(Sense::click()))
            .on_hover_text("Click to switch to email login")
            .clicked()
        {
            self.email_login = !self.email_login;
        }
        ui.add_space(4.0);
    }

    fn username_field(&mut self, ui: &mut Ui) {
        let mut hint = "Username";
        if self.email_login {
            hint = "Email";
        }
        ui.add(
            TextEdit::singleline(&mut self.username)
                .hint_text(hint)
                .margin(vec2(0., 20.))
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);
    }

    fn password_field(&mut self, ui: &mut Ui) {
        ui.add(
            TextEdit::singleline(&mut self.password)
                .hint_text("Password")
                .margin(vec2(0., 20.))
                .password(true)
                .desired_width(f32::INFINITY),
        );
    }

    fn forgot_password(&self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let response = ui.add(Label::new("Forgot password?").sense(Sense::click()));
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if response.clicked() {
                get_app_data().open(Page::ResetPassword);
            }
        });
    }

    fn submit_button(&mut self, ui: &mut Ui) {
        let loading = self.request.loading();
        let email_validation = match self.email_login {
            true => validate_email(&self.username),
            false => true,
        };
        let username_validation = match self.email_login {
            true => true,
            false => validate_username(&self.username),
        };
        let password_validation = validate_password(&self.password);
        let btn = submit_button::render(
            ui,
            loading,
            email_validation && username_validation && password_validation && !loading,
        );
        if btn.clicked() {
            self.request.set_body(match self.email_login {
                true => LoginRequest::Email(LoginWithEmailAndPassword {
                    email: self.username.clone(),
                    password: self.password.clone(),
                }),
                false => LoginRequest::Username(LoginWithUsernameAndPassword {
                    username: self.username.clone(),
                    password: self.password.clone(),
                }),
            });
            self.request.send()
        }
    }
    fn or_continue_with(&self, ui: &mut Ui) {
        centered_line_with_text_widget(ui, "or continue with");
        let style = ui.style().spacing.icon_spacing;

        ui.add_space(8.0);

        // Icon buttons
        ui.horizontal(|ui| {
            let space = (ui.available_width() - ((32. + 2. * style) * 3.)) / 2.0 - style;
            ui.add_space(space);
            //TODO: add login with apple, google, facebook, twitter, github, gitlab
            ui.add(self.apple.clone());
            ui.add(self.google.clone());
            ui.add(self.gh.clone());
            ui.add_space(space);
        });
    }

    fn not_a_member(&self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.label("Not a member?");
            if ui
                .add(Link::new("Register here"))
                .on_hover_text("Click to register")
                .clicked()
            {
                get_app_data().open(Page::SignUp);
            }
        });
    }
}
