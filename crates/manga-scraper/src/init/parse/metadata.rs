use std::{fs::read_to_string, path::Path, sync::Arc};

use scraper_module::MetaDataScraper;

use crate::{
    init::scraper::{Engine, EngineMode},
    processors::{PostMetadataProcessor, PreProcessor},
    runtime::metadata::InterpretedMetadata,
    InitError,
};

use super::selectors;
#[derive(serde::Deserialize)]
struct Metadata {
    request_config: Option<String>,
}
pub fn parse_engine(processors: Vec<&str>) -> Result<Engine, InitError> {
    if processors.contains(&"reqwest_rustls") {
        Engine::new(EngineMode::ReqwestRustlsTls, Default::default())
    } else if processors.contains(&"reqwest") {
        Engine::new(EngineMode::ReqwestNativeTls, Default::default())
    } else if processors.contains(&"curl") {
        Engine::new(EngineMode::Curl, Default::default())
    } else {
        Err(InitError::NoEngineSelected)
    }
}

pub fn parse(
    file: &Path,
    processors: Vec<&str>,
    pre_processors: Vec<Arc<dyn PreProcessor + Sync + Send>>,
    post_processors: Vec<(Arc<dyn PostMetadataProcessor + Sync + Send>, String)>,
) -> crate::Result<Arc<dyn MetaDataScraper>> {
    let text = read_to_string(file)?;
    let selectors = selectors::parse(&text)?;

    Ok(Arc::new(InterpretedMetadata {
        selectors,
        pre_processors,
        engine: parse_engine(processors)?,
        post_processors,
    }))
}
