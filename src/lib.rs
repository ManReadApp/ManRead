mod downloader;
mod error;
mod extractor;
mod pages;
mod services;
//mod tests;

pub use error::ScrapeError;
pub use services::icon::ExternalSite;
pub use services::init;
pub use services::metadata::MetaDataService;
pub use services::multisite::MultiSiteService;
pub use services::search::SearchService;
pub use services::singlesite::SingleSiteService;
