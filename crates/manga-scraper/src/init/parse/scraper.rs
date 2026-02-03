use std::{fs::read_to_string, path::Path, sync::Arc};

use regex::Regex;
use scraper::Selector;
use scraper_module::{Mode, ReaderScraper};
use serde::Deserialize;

use crate::{
    processors::{PostScraperProcessor, PreProcessor},
    runtime::{
        self,
        files::{InterpretedFileScraper, RegexOrSelector},
    },
    InitError,
};

use super::metadata::parse_engine;

#[derive(Deserialize, Debug)]
pub struct Scrape {
    pub mode: Mode,
    pub chapters: Option<Chapter>,
    pub pages: Image,
}

#[derive(Deserialize, Debug)]
pub struct Image {
    pub regex: Option<bool>,
    pub multi: Option<bool>,
    pub container: Option<String>,
    pub items: String,
    pub attr: Option<String>,
    pub fallback_attr: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Chapter {
    #[serde(rename = "url-guess")]
    pub url_guess: Option<bool>,
    pub container: Option<String>,
    pub item: Option<String>,
    pub name: Option<String>,
    pub number: Option<String>,
    pub number_attr: Option<String>,
    pub a: String,
}

pub fn parse(
    file: &Path,
    processors: Vec<&str>,
    pre_processors: Vec<Arc<dyn PreProcessor + Sync + Send>>,
    post_processors: Vec<(Arc<dyn PostScraperProcessor + Sync + Send>, String)>,
) -> crate::Result<Arc<dyn ReaderScraper>> {
    Ok(Arc::new(InterpretedFileScraper::new(
        file,
        processors,
        pre_processors,
        post_processors,
    )?))
}
impl InterpretedFileScraper {
    pub fn new(
        file: &Path,
        processors: Vec<&str>,
        pre_processors: Vec<Arc<dyn PreProcessor + Sync + Send>>,
        post_processors: Vec<(Arc<dyn PostScraperProcessor + Sync + Send>, String)>,
    ) -> crate::Result<Self> {
        let content = read_to_string(file)?;
        let data: Scrape = toml::from_str(&content)?;

        let post_suffix = processors
            .iter()
            .find_map(|v| v.strip_prefix("post scraper-chapter"))
            .map(|v| v.to_owned());

        let convert = |s: Option<String>| match s {
            None => Ok::<Option<Selector>, InitError>(None),
            Some(s) => Ok(Some(Selector::parse(&s)?)),
        };

        let items = match data.pages.regex.unwrap_or_default() {
            true => RegexOrSelector::Regex(Regex::new(&data.pages.items)?),
            false => RegexOrSelector::Selector(Selector::parse(data.pages.items.as_str())?),
        };
        Ok(Self {
            pre_processors,
            post_processors,
            mode: data.mode,
            post_suffix,
            engine: parse_engine(processors)?,
            chapter: match data.chapters {
                Some(chapters) => Some(runtime::files::Chapter {
                    url_guess: chapters.url_guess.unwrap_or_default(),
                    a: Selector::parse(chapters.a.as_str())?,
                    container: convert(chapters.container)?,
                    item: convert(chapters.item)?,
                    name: convert(chapters.name)?,
                    number: convert(chapters.number)?,
                    number_attr: chapters.number_attr,
                }),
                None => None,
            },
            image: runtime::files::Image {
                multi: data.pages.multi.unwrap_or(true),
                container: convert(data.pages.container)?,
                items,
                attr: data.pages.attr,
                fallback_attr: data.pages.fallback_attr,
            },
        })
    }
}
