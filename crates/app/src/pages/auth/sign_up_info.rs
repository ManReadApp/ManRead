use crate::data::user::User;
use crate::fetcher::{upload_image, Complete, UploadFile};
use crate::get_app_data;
use crate::pages::auth::{background, get_background};
use crate::requests::{RegisterRequestFetcher, RequestImpl as _};
use crate::widgets::hover_brackground::HoverBackground;
use crate::widgets::submit_button;
use crate::window_storage::Page;
use api_structure::models::auth::gender::Gender;
use api_structure::models::auth::role::Role;
use api_structure::req::auth::register::RegisterRequest;
use chrono::NaiveDate;
use eframe::{App, Frame};
use egui::{vec2, Color32, Context, Image, Label, Sense, Ui};
use egui_extras::DatePickerButton;
use ethread::ThreadHandler;
use poll_promise::Promise;
use rfd::AsyncFileDialog;

pub struct SignUpInfoPage {
    email: String,
    username: String,
    password: String,
    height: Option<f32>,
    bg: Image<'static>,
    sex: Vec<Image<'static>>,
    icon_name: String,
    icon: Image<'static>,
    gender: usize,
    birth_data: NaiveDate,
    image_upload: Option<Promise<Option<(String, Image<'static>)>>>,
    create_user: RegisterRequestFetcher,
    init: bool,
}

impl SignUpInfoPage {
    pub(crate) fn new(
        email: String,
        username: String,
        password: String,
        icon_name: String,
        icon: Image<'static>,
    ) -> Self {
        let female = Image::new(egui::include_image!("../../assets/gender/female.svg"));
        let male = Image::new(egui::include_image!("../../assets/gender/male.svg"));
        let both = Image::new(egui::include_image!("../../assets/gender/both.svg"));
        Self {
            email,
            username,
            password,
            height: None,
            bg: get_background(),
            sex: vec![female, male, both],
            icon_name,
            icon,
            gender: 0,
            birth_data: NaiveDate::default(),
            image_upload: None,
            create_user: RegisterRequest::fetcher(&get_app_data().url),
            init: false,
        }
    }
}

impl App for SignUpInfoPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            background(&self.bg, ui);
            self.height = Some(self.hover_box(ui, ctx, self.height));
        });
    }
}
impl HoverBackground for SignUpInfoPage {
    fn inner(&mut self, ui: &mut Ui, ctx: &Context) {
        if !self.init {
            self.init = true;
            self.create_user.set_ctx(ctx.clone());
        }
        ui.vertical_centered(|ui| {
            self.header(ui);
            self.gender_selector(ui);
            self.date_picker(ui);
            self.submit(ui);
        });
    }
}

impl SignUpInfoPage {
    fn select_image(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let future = async {
            let file = AsyncFileDialog::new()
                .add_filter("image", &["png", "jpeg", "jpg", "qoi", "webp"])
                .pick_file()
                .await;
            let file = file?;
            let bytes = file.read().await;
            #[cfg(target_arch = "wasm32")]
            let byte_clone = bytes.clone();
            ctx.forget_image("bytes://custom_user_icon");
            let img = Image::from_bytes("bytes://custom_user_icon", bytes);

            #[cfg(not(target_arch = "wasm32"))]
            let data = UploadFile::Path(file.path().to_str().unwrap_or_default().to_string());
            #[cfg(target_arch = "wasm32")]
            let data = UploadFile::Bytes(byte_clone);
            let binding = upload_image(ctx, data, None).await?;
            let v = binding.first()?;
            Some((v.1.clone(), img))
        };
        self.image_upload = Some(ThreadHandler::new_async_ctx(future, Some(ui.ctx())).task);
    }
    fn header(&mut self, ui: &mut Ui) {
        if let Some(v) = self.create_user.result() {
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
        let icon_label = Label::new("Icon").sense(Sense::click());

        if ui.add(icon_label).double_clicked() {
            self.select_image(ui);
        };
        ui.add_space(4.);
        let img = if let Some(img) = &self.image_upload {
            if let Some(img) = img.ready() {
                match img {
                    None => {
                        self.image_upload = None;
                        None
                    }
                    Some((_, img)) => Some(img.clone()),
                }
            } else {
                None
            }
        } else {
            None
        }
        .unwrap_or_else(|| self.icon.clone());
        let img = ui.add(img.sense(Sense::click()).max_width(50.));

        if img.clicked() {
            self.image_upload = None;
            self.select_image(ui);
        }
        // if let Some(v) = &self.temp_imag_loader {
        //     if v.ready().is_none() {
        //         let spinner = Spinner::new();
        //         let pos = (img.rect.max - img.rect.min) / 2. + img.rect.min.to_vec2();
        //         ui.put(
        //             Rect::from_center_size(pos.to_pos2(), Vec2::new(32.0, 32.0)),
        //             spinner,
        //         );
        //     }
        // }
        img.on_hover_text("Click to change your profile picture");
        ui.add_space(20.);
    }

    fn gender_selector(&mut self, ui: &mut Ui) {
        ui.label("Gender");
        ui.add_space(4.);
        let style = ui.style().spacing.icon_spacing;

        let space = (ui.available_width() - ((50. + 2. * style) * 3.)) / 2.0 - style;
        ui.horizontal_top(|ui| {
            ui.add_space(space);
            for (index, img) in self.sex.iter().enumerate() {
                let mut img = img
                    .clone()
                    .sense(Sense::click())
                    .fit_to_exact_size(vec2(50., 50.));

                if index == self.gender {
                    img = img.tint(Color32::DARK_BLUE);
                }
                let img = ui.add(img);
                if img.clicked() {
                    self.gender = index;
                }
            }
        });
        ui.add_space(20.);
    }

    fn date_picker(&mut self, ui: &mut Ui) {
        ui.label("Birth date");
        let date = DatePickerButton::new(&mut self.birth_data);
        let date = date.arrows(false);
        ui.add(date);
        ui.add_space(10.);
    }

    fn submit(&mut self, ui: &mut Ui) {
        let loading = self
            .image_upload
            .as_ref()
            .map(|v| v.ready().is_none())
            .unwrap_or(false);

        let loading = loading || self.create_user.loading();
        let btn = submit_button::render(ui, loading, true);
        if btn.clicked() {
            let icon = if let Some(v) = &self.image_upload {
                if let Some(Some((name, _))) = v.ready() {
                    Some(name.clone())
                } else {
                    None
                }
            } else {
                None
            }
            .unwrap_or_else(|| self.icon_name.clone());
            self.create_user.set_body(RegisterRequest {
                name: self.username.clone(),
                email: self.email.clone(),
                password: self.password.clone(),
                birthdate: self.birth_data,
                gender: Gender::from(self.gender),
                icon_temp_name: icon,
            });
            self.create_user.send();
        }
    }
}
