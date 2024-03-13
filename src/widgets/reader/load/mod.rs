mod multi;
mod single;

use crate::get_app_data;
use crate::widgets::reader::load::multi::multi;
use crate::widgets::reader::load::single::single;
use crate::widgets::reader::overlay::ReaderTranslationArea;
use crate::widgets::reader::progress::Progress;
use crate::widgets::reader::settings::ReadingMode;
use crate::widgets::reader::storage::{
    get_page_resp, get_version_key, ImageStorage, PageData, State,
};
use api_structure::image::MangaReaderImageRequest;
use api_structure::reader::{Action, MangaReaderResponse, ReaderPageResponse, TranslationArea};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use eframe::emath::Vec2;
use eframe::epaint::Color32;
use egui::{Context, Image};
use ethread::ThreadHandler;
use image::EncodableLayout;
use reqwest::header::AUTHORIZATION;
use std::sync::Arc;

/// load images in range
pub fn load_images(
    v: Arc<MangaReaderResponse>,
    hierachy: &[String],
    page_data: &mut PageData,
    chapter: &str,
    ctx: &Context,
    rm: &ReadingMode,
    area: f32,
    progress: &Progress,
    size: Vec2,
    imgs: &mut ImageStorage,
) {
    let reader_page = if let State::ReaderPageResponse(v) =
        get_page_resp(v.clone(), hierachy, page_data, chapter, ctx)
    {
        v
    } else {
        return;
    };
    //let mut before_reader_page = None;
    // let mut after_reader_page = None;

    match rm {
        ReadingMode::Single => single(
            &v,
            hierachy,
            chapter,
            area,
            progress,
            &reader_page,
            1,
            imgs,
            ctx,
        ),
        ReadingMode::Double(_) => single(
            &v,
            hierachy,
            chapter,
            area,
            progress,
            &reader_page,
            2,
            imgs,
            ctx,
        ),
        ReadingMode::Strip => multi(
            &v,
            hierachy,
            chapter,
            area,
            progress,
            &reader_page,
            size.x,
            size.y,
            true,
            imgs,
            ctx,
        ),
        ReadingMode::Row(_) => multi(
            &v,
            hierachy,
            chapter,
            area,
            progress,
            &reader_page,
            size.y,
            size.x,
            false,
            imgs,
            ctx,
        ),
    }
}

fn load_image<'a>(
    v: &Arc<MangaReaderResponse>,
    hierachy: &[String],
    chapter: &str,
    progress: &Progress,
    imgs: &mut ImageStorage,
    rp: &'a Arc<ReaderPageResponse>,
    offset: u32,
    ctx: &Context,
) -> (bool, Action<'a>) {
    let p = rp.get_page(progress.image + offset);
    let cont = match &p {
        Action::Prev => {
            //TODO: load
            true
        }
        Action::Page(p) => {
            if imgs.get(&p.page_id).is_none() {
                let ver = get_version_key(v.get_chapter(chapter).unwrap(), hierachy);
                if let Some(ver) = ver {
                    let data = MangaReaderImageRequest {
                        manga_id: v.manga_id.clone(),
                        chapter_id: chapter.to_string(),
                        version_id: ver,
                        page: progress.image + offset,
                        file_ext: p.ext.clone(),
                    };
                    let page_id = p.page_id.clone();
                    let fetch_trans = p.translation;

                    imgs.insert(
                        page_id.clone(),
                        ThreadHandler::new_async_ctx(
                            async move {
                                let token = format!(
                                    "Bearer {}",
                                    get_app_data().get_access_token().await.unwrap()
                                );
                                let mut translations = vec![];
                                if fetch_trans {
                                    let mut t: Vec<TranslationArea> = serde_json::from_slice(
                                        get_app_data()
                                            .client
                                            .post(
                                                get_app_data()
                                                    .url
                                                    .join("page_translation")
                                                    .unwrap(),
                                            )
                                            .header(AUTHORIZATION, &token)
                                            .json(&data)
                                            .send()
                                            .await
                                            .ok()?
                                            .bytes()
                                            .await
                                            .ok()?
                                            .as_bytes(),
                                    )
                                    .ok()?;
                                    for (index, trans) in t.into_iter().enumerate() {
                                        let back = trans
                                            .background
                                            .split_once(";base64,")
                                            .map(|v| v.1.to_string())
                                            .unwrap_or_else(|| trans.background);
                                        translations.push(ReaderTranslationArea {
                                            translated_text: trans.translated_text,
                                            min_x: trans.min_x,
                                            min_y: trans.min_y,
                                            max_x: trans.max_x,
                                            max_y: trans.max_y,
                                            text_color: Color32::from_rgb(
                                                trans.text_color[0],
                                                trans.text_color[1],
                                                trans.text_color[2],
                                            ),
                                            outline_color: Color32::from_rgb(
                                                trans.outline_color[0],
                                                trans.outline_color[1],
                                                trans.outline_color[2],
                                            ),
                                            background: Image::from_bytes(
                                                format!(
                                                    "bytes://manga_image_{}_overlay_{}",
                                                    page_id, index
                                                ),
                                                STANDARD.decode(back).ok()?,
                                            ),
                                        })
                                    }
                                }

                                let res = get_app_data()
                                    .client
                                    .post(get_app_data().url.join("chapter_page").unwrap())
                                    .header(AUTHORIZATION, token)
                                    .json(&data)
                                    .send()
                                    .await
                                    .ok()?
                                    .bytes()
                                    .await
                                    .ok()?;
                                Some(Arc::new((
                                    Image::from_bytes(
                                        format!("bytes://manga_image_{}", page_id),
                                        res.to_vec(),
                                    ),
                                    translations,
                                )))
                            },
                            Some(ctx),
                        ),
                    )
                }
            }
            false
        }
        Action::Next => {
            //TODO: load
            true
        }
    };
    (cont, p)
}
