use crate::get_app_data;
use api_structure::req::fonts::FontRequest;
use egui::FontFamily;
use ethread::ThreadHandler;

pub fn setup_custom_fonts(c: egui::Context) -> ThreadHandler<Option<()>> {
    let ctx = c.clone();
    let task = async move {
        let mut font_def = egui::FontDefinitions::default();

        let fonts: Vec<String> = get_app_data()
            .client
            .post(get_app_data().url.join("fonts").unwrap())
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;
        for font in &fonts {
            let raw = get_app_data()
                .client
                .post(get_app_data().url.join("font").unwrap())
                .json(&FontRequest::new(font.to_string()))
                .send()
                .await
                .ok()?
                .bytes()
                .await
                .ok()?;
            font_def
                .font_data
                .insert(font.to_string(), egui::FontData::from_owned(raw.to_vec()));
            font_def
                .families
                .entry(FontFamily::Name(font.to_string().into()))
                .or_default()
                .insert(0, font.to_string());
        }
        ctx.set_fonts(font_def);
        *get_app_data().fonts.lock().unwrap() = fonts;
        Some(())
    };
    ThreadHandler::new_async_ctx(task, Some(&c))
}
