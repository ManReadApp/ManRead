use crate::requests::MangaReaderRequestFetcher;
use crate::requests::ReaderPageRequestFetcher;
use std::collections::HashMap;
use std::sync::Arc;

mod image;
mod manga;

pub type PageData = HashMap<String, ReaderPageRequestFetcher>;
use api_structure::resp::reader::pages::ReaderPageResponse;
pub use image::ImageStorage;
pub use manga::get_page_resp;
pub use manga::get_version_key;
pub(crate) struct Storage {
    pub(crate) manga: MangaReaderRequestFetcher,
    pub(crate) page_data: PageData,
    pub(crate) loaded_pages: ImageStorage,
}

#[derive(Clone)]
pub enum State {
    ChapterLoading,
    ChapterError,
    ReaderPageResponse(Arc<ReaderPageResponse>),
    NoChapter,
}
