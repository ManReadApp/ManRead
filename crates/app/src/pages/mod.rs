pub mod add;
pub mod auth;
mod home;
mod info;
mod loading_init_refres;
mod playground;
mod reader;
mod search;

pub use home::HomePage;
pub use info::InfoPage;
pub use loading_init_refres::LoadingInitRefreshPage;
pub use playground::PlaygroundPage;
pub use reader::MangaReaderPage;
pub use search::SearchPage;
