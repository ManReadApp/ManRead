use crate::get_app_data;
use crate::widgets::image_overlay::ImageOverlay;
use api_structure::models::manga::status::Status;
use api_structure::now_timestamp;
use api_structure::req::manga::cover::MangaCoverRequest;
use egui::{Context, Image, Sense};
use ethread::ThreadHandler;
use futures_util::{stream, StreamExt};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

#[derive(Default)]
pub struct CoverStorage {
    items: HashMap<String, CoverTimeStamp>,
}

impl CoverStorage {
    pub fn get_main(&self, manga_id: &str) -> Option<&ImageOverlay> {
        let item = self.items.get(manga_id)?;
        match item.image.task.ready()? {
            None => None,
            Some(v) => Some(v),
        }
    }

    pub fn get_url(&mut self, url: &str, ctx: &Context) -> Option<ImageOverlay> {
        if let Some(item) = self.items.get_mut(url) {
            item.opened = Some(now_timestamp().unwrap());
            return item.image.task.ready()?.clone();
        }
        let new = ThreadHandler::new_async_ctx(Self::download_url(url), Some(ctx));
        self.items.insert(url.to_string(), CoverTimeStamp::new(new));
        None
    }

    pub fn get(
        &mut self,
        manga_id: &str,
        status: &Status,
        ext: &str,
        number: u32,
        ctx: &Context,
    ) -> Option<ImageOverlay> {
        if let Some(item) = self.items.get_mut(manga_id) {
            item.opened = Some(now_timestamp().unwrap());
            return item.image.task.ready()?.clone();
        }
        let new = ThreadHandler::new_async_ctx(
            Self::download_logic(manga_id.to_string(), *status, ext.to_string()),
            Some(ctx),
        );
        self.items
            .insert(manga_id.to_string(), CoverTimeStamp::new(new));
        None
    }

    fn new(data: HashMap<String, Option<ImageOverlay>>) -> Self {
        Self {
            items: data
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        CoverTimeStamp::new_manual(ThreadHandler::new(|| value)),
                    )
                })
                .collect(),
        }
    }

    pub fn append(&mut self, other: Self) {
        for other in other.items {
            self.items.insert(other.0, other.1);
        }
    }

    pub async fn download_many(ids: HashMap<String, (Status, String, u32)>) -> Self {
        let reqs = ids
            .into_iter()
            .map(|(manga_id, (status, ext, number))| Self::download(manga_id, status, ext));
        let v: HashMap<String, Option<ImageOverlay>> = stream::iter(reqs)
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .map(|v| (v.0, Some(v.1)))
            .collect::<HashMap<_, _>>();
        Self::new(v)
    }

    async fn download(
        manga_id: String,
        status: Status,
        ext: String,
    ) -> Option<(String, ImageOverlay)> {
        Some((
            manga_id.clone(),
            Self::download_logic(manga_id, status, ext).await?,
        ))
    }

    fn download_url(url: &str) -> impl Future<Output = Option<ImageOverlay>> + Sized {
        let req = get_app_data().client.get(url).header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.3");
        let uri = format!("url://{}", url);
        async move {
            let bytes = req.send().await.ok()?.bytes().await.ok()?;
            let img = Image::from_bytes(uri, bytes.to_vec()).sense(Sense::click());
            Some(ImageOverlay::ongoing(img))
        }
    }

    async fn download_logic(manga_id: String, status: Status, ext: String) -> Option<ImageOverlay> {
        let app = get_app_data();
        let token = format!("Bearer {}", app.get_access_token().await.unwrap());
        let bytes = app
            .client
            .post(app.url.join("cover").unwrap())
            .header(AUTHORIZATION, token)
            .json(&MangaCoverRequest {
                manga_id: manga_id.clone(),
                file_ext: ext,
            })
            .send()
            .await
            .ok()?
            .bytes()
            .await
            .ok()?;
        let img = Image::from_bytes(format!("cover://{}", manga_id), bytes.to_vec())
            .sense(Sense::click());

        Some(match status {
            Status::Dropped => ImageOverlay::dropped(img),
            Status::Hiatus => ImageOverlay::hiatus(img),
            Status::Ongoing => ImageOverlay::ongoing(img),
            Status::Completed => ImageOverlay::completed(img),
            Status::Upcoming => ImageOverlay::upcoming(img),
        })
    }
}

struct CoverTimeStamp {
    opened: Option<Duration>,
    image: ThreadHandler<Option<ImageOverlay>>,
}

impl CoverTimeStamp {
    fn new_manual(image: ThreadHandler<Option<ImageOverlay>>) -> Self {
        Self {
            opened: None,
            image,
        }
    }

    fn new(image: ThreadHandler<Option<ImageOverlay>>) -> Self {
        Self {
            opened: Some(now_timestamp().unwrap()),
            image,
        }
    }
}
