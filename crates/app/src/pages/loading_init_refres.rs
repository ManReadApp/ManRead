use crate::data::user::User;
use crate::get_app_data;
use crate::widgets::centered_spinner;
use crate::window_storage::Page;
use eframe::{App, Frame};
use egui::{Context, Image};
use ethread::ThreadHandler;
use log::info;
use reqwest::RequestBuilder;
use std::borrow::Cow;

pub struct LoadingInitRefreshPage {
    data: ThreadHandler<Option<(User, Image<'static>)>>,
    ctx_set: bool,
}

async fn get_spinner(rb: RequestBuilder) -> Option<Image<'static>> {
    let v = rb.send().await.ok()?.bytes().await.ok()?;
    Some(Image::from_bytes(Cow::from("bytes://spinner"), v.to_vec()))
}

impl LoadingInitRefreshPage {
    pub fn new() -> Self {
        let page = async {
            let data = get_app_data();
            let req = get_spinner(data.client.post(data.url.join("spinner").unwrap())).await?;
            let token = User::load_token();
            info!("Loading token");
            if let Ok(token) = token {
                info!("Found token");
                let tokens = User::get_updated_tokens(&token).await;
                info!("updated token");
                if let Some(jwts) = tokens {
                    return User::new(jwts).map(|v| (v, req));
                }
            }
            None
        };
        Self {
            data: ThreadHandler::new_async(page),
            ctx_set: false,
        }
    }
}

impl App for LoadingInitRefreshPage {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        if !self.ctx_set {
            self.ctx_set = true;
            self.data.set_context(ctx.clone());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(v) = self.data.task.ready() {
                match v {
                    None => {
                        get_app_data().change(Page::SignIn, Page::all());
                        ctx.request_repaint();
                        return;
                    }
                    Some((user, image)) => {
                        let data = get_app_data();
                        *data.spinner.lock().unwrap() = Some(image.clone());
                        data.set_user_data(user.clone());
                        #[cfg(feature = "dev")]
                        let page = Page::Playground;
                        #[cfg(not(feature = "dev"))]
                        let page = if Role::NotVerified == data.get_user_data().unwrap().role {
                            Page::VerifyAccount
                        } else {
                            Page::Home
                        };
                        data.change(page, Page::all());
                        ctx.request_repaint();
                        return;
                    }
                }
            }
            centered_spinner::render(ui)
        });
    }
}
