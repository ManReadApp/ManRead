use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use post::{
    add_base::AddBasePostProcessor, array_to_map::ArrayToMapPostProcessor,
    filter_duplicates::FilterDuplicatesProcessor, filter_empty::FilterEmptyPostProcessor,
    flatten::FlattenPostProcessor, join::JoinProcessor, json::JsonPostProcessor,
    json_display::JsonDispayProcessor, json_flatten::JsonFlattenPostProcessor,
    json_string_string::JsonStringStringPostProcessor, json_take::TakeJsonPostProcessor,
    next_image::NextImageProcessor, prefix_key::PrefixKeyProcessor,
    remove_attr::RemoveAttrPostProcessor, remove_attrs::RemoveAttrs, split::SplitProcessor,
    strip::StripSuffixPostProcessor, strip_string::StripSuffixStringProcessor,
};
use scraper_module::{ScrapedData, ScrapedSearchResponse};

use crate::{init::scraper::Engine, ScrapeError};

mod post;

pub struct Processors {
    pub post_search: HashMap<String, Arc<dyn PostSearchProcessor + Sync + Send>>,
    pub post_meta: HashMap<String, Arc<dyn PostMetadataProcessor + Sync + Send>>,
    pub post_scrape: HashMap<String, Arc<dyn PostScraperProcessor + Sync + Send>>,
    pub pre: HashMap<String, Arc<dyn PreProcessor + Sync + Send>>,
}

pub enum Target {
    Metadata,
    Search,
    Scraper,
}

impl TryFrom<&str> for Target {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "metadata" => Ok(Self::Metadata),
            "search" => Ok(Self::Search),
            "scraper" => Ok(Self::Scraper),
            _ => Err(()),
        }
    }
}

macro_rules! register_post {
    ($name:ident, $ty:ty, $target:path, $map:ident) => {
        impl Processors {
            pub fn $name(&self, name: &str) -> Option<(Arc<$ty>, String)> {
                let name = name.trim();
                if let Some(v) = name.strip_prefix("postprocessor ") {
                    let (target, info) = v.split_once(" ").unwrap();
                    let _ = match Target::try_from(target) {
                        Ok(v) => match v {
                            $target => true,
                            _ => return None,
                        },
                        Err(_) => return None,
                    };
                    let (id, query) = info.split_once(" ").unwrap_or((info, ""));
                    return self.$map.get(id).cloned().map(|v| (v, query.to_owned()));
                }
                None
            }
        }
    };
}

register_post!(
    get_post_processor_meta,
    dyn PostMetadataProcessor + Sync + Send,
    Target::Metadata,
    post_meta
);

register_post!(
    get_post_processor_search,
    dyn PostSearchProcessor + Sync + Send,
    Target::Search,
    post_search
);

register_post!(
    get_post_processor_scraper,
    dyn PostScraperProcessor + Sync + Send,
    Target::Scraper,
    post_scrape
);

impl Processors {
    pub fn new() -> Self {
        Self {
            pre: get_pre_processors(),
            post_meta: get_post_processors_meta(),
            post_search: get_post_processors_search(),
            post_scrape: get_post_processors_scrape(),
        }
    }

    pub fn get_pre_processor(
        &self,
        name: &str,
    ) -> Option<(Arc<dyn PreProcessor + Sync + Send>, Target)> {
        let name = name.trim();
        if let Some(v) = name.strip_prefix("preprocessor ") {
            let (target, id) = v.split_once(" ").unwrap();
            let target = match Target::try_from(target) {
                Ok(v) => v,
                Err(_) => return None,
            };
            return self.pre.get(id.trim()).cloned().map(|v| (v, target));
        }
        None
    }
}

pub trait PostSearchProcessor {
    fn name(&self) -> &str;
    fn process(
        &self,
        url: &str,
        data: Vec<ScrapedSearchResponse>,
        target: &str,
    ) -> Vec<ScrapedSearchResponse>;
}

pub trait PostMetadataProcessor {
    fn name(&self) -> &str;
    fn process(&self, data: &mut BTreeMap<String, ScrapedData>, target: &str);
}

pub trait PostScraperProcessor {
    fn name(&self) -> &str;
    fn process(&self, data: Vec<String>, target: &str) -> Vec<String>;
}

#[async_trait]
pub trait PreProcessor {
    fn name(&self) -> &str;
    async fn process(&self, url: &str, engine: &Engine) -> Result<String, ScrapeError>;
}

pub mod pre;

fn get_pre_processors() -> HashMap<String, Arc<dyn PreProcessor + Sync + Send>> {
    let mut map = HashMap::new();
    map.insert(
        pre::asura::AsuraPreProcessor.name().to_owned(),
        Arc::new(pre::asura::AsuraPreProcessor) as Arc<dyn PreProcessor + Sync + Send>,
    );
    map
}

fn get_post_processors_search() -> HashMap<String, Arc<dyn PostSearchProcessor + Sync + Send>> {
    let mut map = HashMap::new();
    macro_rules! insert_processor {
        ($processor:ty) => {{
            let instance = <$processor>::default();
            let item = Arc::new(instance) as Arc<dyn PostSearchProcessor + Sync + Send>;
            map.insert(item.name().to_owned(), item);
        }};
    }

    insert_processor!(AddBasePostProcessor);
    insert_processor!(RemoveAttrPostProcessor);
    insert_processor!(FilterEmptyPostProcessor);
    insert_processor!(NextImageProcessor);
    insert_processor!(RemoveAttrs);
    map
}
fn get_post_processors_scrape() -> HashMap<String, Arc<dyn PostScraperProcessor + Sync + Send>> {
    let mut map = HashMap::new();
    macro_rules! insert_processor {
        ($processor:ty) => {{
            let instance = <$processor>::default();
            let item = Arc::new(instance) as Arc<dyn PostScraperProcessor + Sync + Send>;
            map.insert(item.name().to_owned(), item);
        }};
    }

    insert_processor!(JsonPostProcessor);
    insert_processor!(TakeJsonPostProcessor);
    insert_processor!(JsonStringStringPostProcessor);
    insert_processor!(JsonFlattenPostProcessor);
    insert_processor!(JsonDispayProcessor);
    insert_processor!(RemoveAttrs);
    insert_processor!(SplitProcessor);

    map
}

fn get_post_processors_meta() -> HashMap<String, Arc<dyn PostMetadataProcessor + Sync + Send>> {
    let mut map = HashMap::new();
    macro_rules! insert_processor {
        ($processor:ty) => {{
            let instance = <$processor>::default();
            let item = Arc::new(instance) as Arc<dyn PostMetadataProcessor + Sync + Send>;
            map.insert(item.name().to_owned(), item);
        }};
    }

    insert_processor!(ArrayToMapPostProcessor);
    insert_processor!(FlattenPostProcessor);
    insert_processor!(StripSuffixPostProcessor);
    insert_processor!(StripSuffixStringProcessor);
    insert_processor!(PrefixKeyProcessor);
    insert_processor!(SplitProcessor);
    insert_processor!(FilterDuplicatesProcessor);
    insert_processor!(JoinProcessor);
    insert_processor!(NextImageProcessor);

    map
}
