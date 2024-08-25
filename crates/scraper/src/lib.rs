mod downloader;
mod error;
mod extractor;
mod pages;
mod services;
mod tests;

pub fn find_workspace_root() -> Option<PathBuf> {
    let mut current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_str()?.to_string();

    loop {
        let cargo_toml_path = format!("{}/Cargo.toml", current_dir);
        if let Ok(content) = read_to_string(&cargo_toml_path) {
            if content.contains("[workspace]") {
                return Some(PathBuf::from(current_dir));
            }
        }

        let parent_dir = match Path::new(&current_dir).parent() {
            Some(dir) => dir.to_str()?.to_string(),
            None => break,
        };

        if parent_dir == current_dir {
            break;
        }

        current_dir = parent_dir;
    }

    None
}

pub use error::ScrapeError;
pub use services::icon::ExternalSite;
pub use services::init;
pub use services::metadata::MetaDataService;
pub use services::multisite::MultiSiteService;
pub use services::search::SearchService;
pub use services::singlesite::SingleSiteService;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
