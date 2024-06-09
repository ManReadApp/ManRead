use crate::get_app_data;
use crate::widgets::reader::overlay::ReaderTranslationArea;
use api_structure::now_timestamp;
use egui::{include_image, Context, Image};
use ethread::ThreadHandler;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
type ImageWithTranslations = Option<Arc<(Image<'static>, Vec<ReaderTranslationArea>)>>;

pub struct ImageStorage {
    hashmap: HashMap<String, ImageStore>,
    pub(crate) loading: Arc<(Image<'static>, Vec<ReaderTranslationArea>)>,
    pub(crate) error: Arc<(Image<'static>, Vec<ReaderTranslationArea>)>,
}

impl Default for ImageStorage {
    fn default() -> Self {
        Self {
            hashmap: Default::default(),
            loading: Arc::new((
                get_app_data().spinner.lock().unwrap().clone().unwrap(),
                vec![],
            )),
            error: Arc::new((
                Image::from(include_image!("../../../assets/error.gif")),
                vec![],
            )),
        }
    }
}

impl ImageStorage {
    pub(crate) fn get(&mut self, s: &str) -> Option<&ImageStore> {
        if let Some(v) = self.hashmap.get_mut(s) {
            v.last_checked = now_timestamp().unwrap();
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn insert(&mut self, key: String, image: ThreadHandler<ImageWithTranslations>) {
        self.hashmap.insert(
            key,
            ImageStore {
                last_checked: now_timestamp().unwrap(),
                req: image,
            },
        );
    }

    pub fn clean(&mut self, ctx: &Context) {
        if let Some(v) = self.hashmap.values().map(|v| v.last_checked).max() {
            let mut dispose = vec![];
            let dispose_timestamp = v - Duration::from_secs(1);
            for (key, item) in &self.hashmap {
                if item.last_checked < dispose_timestamp {
                    dispose.push(key.clone());
                }
            }
            for dispose in dispose {
                if let Some(v) = self.hashmap.remove(&dispose) {
                    if let Some(v) = v.req.task.ready() {
                        if let Some(v) = v {
                            for image in &v.1 {
                                if let Some(v) = image.background.source().uri() {
                                    ctx.forget_image(v);
                                }
                            }
                            if let Some(v) = v.0.source().uri() {
                                ctx.forget_image(v);
                            }
                        }
                    } else {
                        v.req.dispose_of_thread()
                    }
                }
            }
        }
    }
}

pub struct ImageStore {
    last_checked: Duration,
    pub(crate) req: ThreadHandler<ImageWithTranslations>,
}
