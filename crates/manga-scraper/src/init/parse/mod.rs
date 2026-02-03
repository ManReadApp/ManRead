mod metadata;
mod register;
mod scraper;
mod search;
pub mod selectors;
pub use search::InterpretedSearch;

use std::{
    collections::HashMap,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::Arc,
};

use scraper_module::{MetaDataScraper, ReaderScraper, Register, SearchScraper};

use crate::processors::{Processors, Target};

pub fn parse(
    root: &Path,
) -> crate::Result<(
    Vec<(String, Arc<dyn Register>)>,
    Vec<(String, Arc<dyn SearchScraper>)>,
    Vec<(String, Arc<dyn MetaDataScraper>)>,
    Vec<(String, Arc<dyn ReaderScraper>)>,
)> {
    let mut files = files(root);
    let filter = files.remove("filter").unwrap_or_default();
    let search = files.remove("search").unwrap_or_default();
    let metadata = files.remove("metadata").unwrap_or_default();
    let scraper = files.remove("scraper").unwrap_or_default();

    let headers = files.remove("header").unwrap_or_default();
    let mut imgs = files
        .into_iter()
        .flat_map(|(ty, v)| v.into_iter().map(move |v| (ty, v)))
        .map(|v| {
            let name = v.1.file_name().and_then(|v| v.to_str()).unwrap().to_owned();
            (
                name.strip_suffix(format!(".{}", v.0).as_str())
                    .unwrap_or(&name)
                    .to_owned(),
                (v.0, v.1),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut scrapers = vec![];
    for filter in filter {
        let uri = filter
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .rsplit_once(".")
            .unwrap()
            .0;
        let scraper = register::parse(&filter, uri, &mut imgs)?;
        scrapers.push((uri.to_owned(), scraper));
    }
    let mut searches = vec![];
    let processors = Processors::new();

    let mut metadatas = vec![];
    for search in search {
        let uri = search
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .rsplit_once(".")
            .unwrap()
            .0;
        let item = scrapers
            .iter()
            .find(|v| v.0 == uri)
            .map(|b| b.1.get_used_processor_names())
            .unwrap_or_default();
        let post_search = item
            .iter()
            .filter_map(|v| processors.get_post_processor_search(v))
            .collect::<Vec<_>>();

        searches.push((uri.to_owned(), search::parse(&search, item, post_search)?));
    }

    let mut readers = vec![];

    for scraper in scraper {
        let uri = scraper
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .rsplit_once(".")
            .unwrap()
            .0;
        let item = scrapers
            .iter()
            .find(|v| v.0 == uri)
            .map(|b| b.1.get_used_processor_names())
            .unwrap_or_default();
        let pre = item
            .iter()
            .filter_map(|v| processors.get_pre_processor(v))
            .collect::<Vec<_>>();
        let pre_meta = pre
            .iter()
            .filter_map(|(tr, t)| match t {
                Target::Metadata => Some(tr.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let post_scrape = item
            .iter()
            .filter_map(|v| processors.get_post_processor_scraper(v))
            .collect::<Vec<_>>();
        readers.push((
            uri.to_owned(),
            scraper::parse(&scraper, item, pre_meta, post_scrape)?,
        ));
    }

    for metadata in metadata {
        let uri = metadata
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .rsplit_once(".")
            .unwrap()
            .0;
        let item = scrapers
            .iter()
            .find(|v| v.0 == uri)
            .map(|b| b.1.get_used_processor_names())
            .unwrap_or_default();
        let post_meta = item
            .iter()
            .filter_map(|v| processors.get_post_processor_meta(v))
            .collect::<Vec<_>>();
        let pre = item
            .iter()
            .filter_map(|v| processors.get_pre_processor(v))
            .collect::<Vec<_>>();
        let pre_meta = pre
            .iter()
            .filter_map(|(tr, t)| match t {
                Target::Metadata => Some(tr.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let metadata = metadata::parse(&metadata, item, pre_meta, post_meta)?;
        metadatas.push((uri.to_owned(), metadata));
    }
    Ok((scrapers, searches, metadatas, readers))
}

pub fn files(root: &Path) -> HashMap<&'static str, Vec<PathBuf>> {
    let ext = [
        "filter", "search", "metadata", "scraper", "header", "qoi", "png", "jpg", "jpeg", "webp",
        "svg", "gif", "ico", "afiv",
    ];
    read_dir(root.join("external"))
        .map(|v| {
            v.filter_map(|v| v.ok())
                .filter(|v| v.path().is_file())
                .filter_map(|v| match v.path().extension().and_then(|v| v.to_str()) {
                    None => None,
                    Some(f_ext) => ext.into_iter().find(|v| v == &f_ext).map(|a| (a, v.path())),
                })
                .fold(HashMap::new(), |mut acc, (key, value)| {
                    acc.entry(key).or_insert_with(Vec::new).push(value);
                    acc
                })
        })
        .unwrap_or_default()
}
