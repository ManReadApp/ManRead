use std::{collections::HashMap, path::Path, sync::Arc};

use scraper_module::{MetaDataScraper, ReaderScraper, Register, SearchScraper};

mod load_dyn;
pub mod parse;
pub mod scraper;

pub struct Service {
    pub uri: String,
    pub register: Arc<dyn Register>,
    pub searchers: Option<Arc<dyn SearchScraper>>,
    pub metadata: Option<Arc<dyn MetaDataScraper>>,
    pub reader: Option<Arc<dyn ReaderScraper>>,
}

pub struct Services {
    pub services: Vec<Service>,
}

impl Services {
    pub fn new(services: Vec<Service>) -> Self {
        Self { services }
    }
    pub fn get_by_uri(&self, uri: &str) -> Option<&Service> {
        self.services.iter().find(|v| v.uri == uri)
    }
    pub fn get(&self, url: &str) -> Option<&Service> {
        self.services.iter().find(|v| v.register.url_matches(url))
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }
}

pub fn register(root: &Path) -> crate::Result<Services> {
    let (registers, searchers, metadata, reader) = parse::parse(root)?;
    let mut searchers = searchers.into_iter().collect::<HashMap<_, _>>();
    let mut metadata = metadata.into_iter().collect::<HashMap<_, _>>();
    let mut reader = reader.into_iter().collect::<HashMap<_, _>>();
    let dylibs = load_dyn::load_dyn(root);
    for dylib in dylibs {
        if let Some(search) = dylib.search {
            searchers.insert(dylib.uri.to_owned(), search);
        }

        if let Some(meta) = dylib.metadata {
            metadata.insert(dylib.uri.to_owned(), meta);
        }
        if let Some(read) = dylib.reader {
            reader.insert(dylib.uri.to_owned(), read);
        }
    }
    let mut services = vec![];
    for (uri, register) in registers {
        services.push(Service {
            register,
            searchers: searchers.remove(&uri),
            metadata: metadata.remove(&uri),
            reader: reader.remove(&uri),
            uri,
        })
    }
    Ok(Services::new(services))
}
