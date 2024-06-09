#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::app::TemplateApp;
use crate::data::shared_data::SharedData;
#[cfg(target_arch = "wasm32")]
use egui::{include_image, vec2, Image, Vec2};
#[cfg(not(target_arch = "wasm32"))]
use fern::colors::{Color, ColoredLevelConfig};
use std::sync::Arc;
use std::time::SystemTime;

mod app;
mod data;
mod fetcher;
mod fonts;
mod pages;
mod util;
mod widgets;
mod window_storage;

static mut APP_DATA: Option<Arc<SharedData>> = None;
fn get_app_data() -> &'static Arc<SharedData> {
    unsafe { APP_DATA.as_ref().unwrap() }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eframe::Result<()> {
    unsafe { APP_DATA = Some(Arc::new(SharedData::new())) };
    let colors = ColoredLevelConfig::default()
        .trace(Color::Cyan)
        .debug(Color::Blue)
        .info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([380.0, 400.0])
            .with_decorations(false)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/logo.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "ManRead",
        native_options,
        Box::new(|_| Ok(Box::<TemplateApp>::default())),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    unsafe { APP_DATA = Some(Arc::new(SharedData::new())) };

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::<TemplateApp>::default()),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[cfg(target_arch = "wasm32")]
fn get_window_dimensions() -> Vec2 {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let width = body.client_width() as f32;
    let height = body.client_height() as f32;

    vec2(width, height)
}
