use async_trait::async_trait;
use reqwest::Url;

use crate::{init::scraper::Engine, processors::PreProcessor, ScrapeError};

pub struct AsuraPreProcessor;

#[async_trait]
impl PreProcessor for AsuraPreProcessor {
    fn name(&self) -> &str {
        "asura"
    }
    async fn process(&self, url: &str, engine: &Engine) -> Result<String, ScrapeError> {
        process_url(engine, url).await
    }
}

async fn process_url(engine: &Engine, url: &str) -> Result<String, ScrapeError> {
    if url.contains("&name=") {
        let html = engine.request_str(true, &url).await?;
        get_first_url(&Url::parse(&url)?.origin().ascii_serialization(), &html)
            .ok_or(ScrapeError::NodeNotFound)
    } else {
        Ok(url.to_owned())
    }
}

pub fn get_first_url(base: &str, input: &str) -> Option<String> {
    let pattern = r#"<a\s+href="series/([^"]+)">"#;

    let regex = regex::Regex::new(pattern).expect("Failed to compile regex");

    if let Some(captures) = regex.captures(input) {
        if let Some(url) = captures.get(1) {
            return Some(format!("{base}/series/{}", url.as_str()));
        }
    }

    None
}
