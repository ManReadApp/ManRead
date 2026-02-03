use std::{collections::BTreeMap, sync::Arc};

use scraper_module::{MetaDataScraper, ScrapedData, ScraperError};

use crate::{
    init::{parse::selectors::MySelector, scraper::Engine},
    processors::{PostMetadataProcessor, PreProcessor},
    ScrapeError,
};

use super::selector::{flatten, OutData};

pub struct InterpretedMetadata {
    pub engine: Engine,
    pub selectors: Vec<MySelector>,
    pub pre_processors: Vec<Arc<dyn PreProcessor + Sync + Send>>,
    pub post_processors: Vec<(Arc<dyn PostMetadataProcessor + Sync + Send>, String)>,
}

impl From<ScrapeError> for ScraperError {
    fn from(value: ScrapeError) -> Self {
        match value {
            ScrapeError::NodeNotFound => ScraperError::NodeNotFound,
            ScrapeError::Cloudflare => ScraperError::Cloudflare,
            ScrapeError::UrlParseError(_) => ScraperError::InvalidUrl,
            ScrapeError::Reqwest(error) => ScraperError::Reqwest(error.to_string()),
        }
    }
}
#[async_trait::async_trait]
impl MetaDataScraper for InterpretedMetadata {
    async fn scrape_metadata(
        &self,
        url: &str,
    ) -> Result<BTreeMap<String, ScrapedData>, ScraperError> {
        let mut new_url = url.to_owned();
        for processor in self.pre_processors.iter() {
            new_url = processor.process(&new_url, &self.engine).await.unwrap();
        }
        let html = self.engine.request_str(true, &new_url).await?;
        let res = self
            .selectors
            .iter()
            .map(|selector| selector.run(&html).unwrap())
            .map(|(a, b)| OutData::Tuple((a, Box::new(b))))
            .collect::<Vec<_>>();
        let mut data = flatten(OutData::Array(res)).as_map().unwrap();
        data.insert("url".to_string(), ScrapedData::Str(url.to_string()));
        if url != new_url {
            data.insert("new_url".to_string(), ScrapedData::Str(new_url));
        }
        for (processor, target) in self.post_processors.iter() {
            processor.process(&mut data, target);
            println!("name {}", processor.name());
        }
        Ok(data)
    }
}
