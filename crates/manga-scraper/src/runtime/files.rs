use std::{collections::HashMap, sync::Arc};

use natord::compare;
use regex::Regex;
use scraper::{Html, Selector};
use scraper_module::{Mode, ReaderScraper, ScrapedChapter, ScraperError, ScraperResult};
use url::Url;

use crate::{
    init::scraper::Engine,
    processors::{PostScraperProcessor, PreProcessor},
};

pub struct InterpretedFileScraper {
    pub engine: Engine,
    pub post_suffix: Option<String>,
    pub mode: Mode,
    pub chapter: Option<Chapter>,
    pub pre_processors: Vec<Arc<dyn PreProcessor + Sync + Send>>,
    pub post_processors: Vec<(Arc<dyn PostScraperProcessor + Sync + Send>, String)>,
    pub image: Image,
}

pub struct Image {
    pub container: Option<Selector>,
    pub items: RegexOrSelector,
    pub multi: bool,
    pub attr: Option<String>,
    pub fallback_attr: Option<String>,
}

pub enum RegexOrSelector {
    Regex(Regex),
    Selector(Selector),
}

pub struct Chapter {
    pub url_guess: bool,
    pub container: Option<Selector>,
    pub item: Option<Selector>,
    pub name: Option<Selector>,
    pub number: Option<Selector>,
    pub number_attr: Option<String>,
    pub a: Selector,
}

#[async_trait::async_trait]
impl ReaderScraper for InterpretedFileScraper {
    fn multi(&self, _: &str) -> Mode {
        self.mode
    }
    async fn download_file(&self, url: &str) -> ScraperResult<Vec<u8>> {
        Ok(self.engine.request(true, url).await?)
    }

    /// Returns urls to the pages
    async fn scrape_pages(&self, url: &str) -> ScraperResult<Vec<String>> {
        let html = self.engine.request_str(true, &url).await?;
        let out: Result<Vec<String>, ScraperError> = match &self.image.items {
            RegexOrSelector::Regex(re) => {
                let items = re
                    .captures_iter(&html)
                    .filter_map(|caps| caps.name("value").map(|v| (caps.name("key"), v)))
                    .map(|(k, v)| (k.map(|k| k.as_str().to_owned()), v.as_str().to_owned()))
                    .collect::<Vec<_>>();
                let map = items.iter().all(|v| v.0.is_some());
                let mut items = match map {
                    true => {
                        let mut items = items
                            .into_iter()
                            .map(|(k, v)| (k.unwrap(), v))
                            .collect::<HashMap<_, _>>()
                            .into_iter()
                            .collect::<Vec<_>>();
                        items.sort_by(|(a, _), (b, _)| compare(a, b));
                        items.into_iter().map(|v| v.1).collect::<Vec<_>>()
                    }
                    false => items.into_iter().map(|v| v.1).collect(),
                };
                if !self.image.multi {
                    items = vec![items.into_iter().next().ok_or(ScraperError::NodeNotFound)?];
                }
                Ok(items)
            }
            RegexOrSelector::Selector(items) => {
                let document = Html::parse_document(&html);
                let mut v: Vec<_> = match &self.image.container {
                    Some(v) => document
                        .select(v)
                        .next()
                        .ok_or(ScraperError::NodeNotFound)?
                        .select(items)
                        .collect(),
                    None => document.select(items).collect(),
                };
                if !self.image.multi {
                    v = vec![v.into_iter().next().ok_or(ScraperError::NodeNotFound)?];
                }
                Ok(v.into_iter()
                    .filter_map(|v| {
                        let out = v.attr(match &self.image.attr {
                            Some(v) => v.as_str(),
                            None => "src",
                        });
                        match (out, &self.image.fallback_attr) {
                            (None, Some(attr)) => v.attr(attr),
                            (v, _) => v,
                        }
                    })
                    .map(|v| v.trim().to_owned())
                    .collect())
            }
        };
        let mut out = out?;
        for (processor, target) in &self.post_processors {
            out = processor.process(out, target);
        }

        Ok(out)
    }

    /// Returns array of chapters
    async fn scrape_chapters(&self, url: &str) -> ScraperResult<Vec<ScrapedChapter>> {
        let chapter = match &self.chapter {
            Some(v) => v,
            None => return Err(ScraperError::Unimplemented),
        };
        let mut new_url = url.to_owned();
        for processor in self.pre_processors.iter() {
            new_url = processor.process(&new_url, &self.engine).await.unwrap();
        }
        let mut get = true;
        if let Some(suffix) = &self.post_suffix {
            let suffix = suffix.as_str().trim();
            if !new_url.ends_with("/") {
                new_url.push('/');
            }

            if suffix.starts_with("/") {
                if let Ok(url) = Url::parse(&new_url) {
                    new_url = url
                        .join(suffix)
                        .map(|v| v.as_str().to_owned())
                        .unwrap_or(new_url);
                }
            } else {
                new_url.extend(suffix.chars());
            }
            get = false;
        }
        let html = self.engine.request_str(get, &new_url).await?;
        let url = Url::parse(&new_url).unwrap();
        let document = Html::parse_document(&html);
        let doc2 = match &chapter.container {
            Some(cont) => Some(
                document
                    .select(&cont)
                    .next()
                    .ok_or(ScraperError::NodeNotFound)?,
            ),
            None => None,
        };
        let items = match &chapter.item {
            Some(v) => {
                let items: Vec<_> = match doc2 {
                    Some(document) => document.select(v).collect(),
                    None => document.select(v).collect(),
                };

                items
                    .into_iter()
                    .map(|v| {
                        v.select(&chapter.a)
                            .next()
                            .and_then(|v| v.attr("href"))
                            .ok_or(ScraperError::AttrNotFound)
                            .map(|href| (v, href.to_owned()))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
            None => {
                let a: Vec<_> = match doc2 {
                    Some(document) => document.select(&chapter.a).collect(),
                    None => document.select(&chapter.a).collect(),
                };
                a.into_iter()
                    .map(|v| {
                        v.attr("href")
                            .ok_or(ScraperError::AttrNotFound)
                            .map(|href| (v, href.to_owned()))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        items
            .into_iter()
            .map(|(container, url_rel)| {
                let name = match &chapter.name {
                    Some(name) => container
                        .select(&name)
                        .next()
                        .ok_or(ScraperError::NodeNotFound)?
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_owned(),
                    None => container.text().collect::<String>().trim().to_owned(),
                };
                let url = url.join(&url_rel).map(|v| v.to_string()).unwrap_or(url_rel);
                let number = match &chapter.number {
                    Some(s) => {
                        let elem = container
                            .select(&s)
                            .next()
                            .ok_or(ScraperError::NodeNotFound)?;
                        let name = match &chapter.number_attr {
                            Some(attr) => elem
                                .attr(attr)
                                .ok_or(ScraperError::AttrNotFound)?
                                .to_owned(),
                            None => elem.text().collect::<String>().trim().to_owned(),
                        };
                        let name = name.strip_prefix("c-").unwrap_or(&name);
                        name.parse::<f64>()
                            .map_err(|_| ScraperError::InvalidChapterNum(name.to_owned()))
                    }
                    None => match &chapter.number_attr {
                        Some(attr) => {
                            let name = container.attr(&attr).ok_or(ScraperError::AttrNotFound)?;
                            let name = name.strip_prefix("c-").unwrap_or(&name);
                            name.parse::<f64>()
                                .map_err(|_| ScraperError::InvalidChapterNum(name.to_owned()))
                        }
                        None => guess_episode(&match chapter.url_guess {
                            true => &url,
                            false => &name,
                        }),
                    },
                }?;
                Ok(ScrapedChapter {
                    tags: Vec::new(),
                    names: vec![strip_chapter_prefix(&name).unwrap_or(name)]
                        .into_iter()
                        .filter(|v| v.len() > 0)
                        .collect(),
                    chapter: number,
                    url,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

pub fn strip_chapter_prefix(s: &str) -> Option<String> {
    let re = Regex::new(r"(?i)chapter\s+\d+(\.\d+)?\s*[:-]?\s*(.*)").unwrap();
    let re2 = Regex::new(r"(?i)ch\.\s*\d+(\.\d+)?\s*[:-]?\s*(.*)").unwrap();
    let re3 = Regex::new(r"第\d+(\.\d+)?\s*[:-]?\s*(.*)").unwrap();

    if let Some(cap) = re.captures(&s) {
        return Some(cap.get(2).map_or("", |m| m.as_str()).trim().to_string());
    } else if let Some(cap) = re2.captures(&s) {
        return Some(cap.get(2).map_or("", |m| m.as_str()).trim().to_string());
    } else if let Some(cap) = re3.captures(s) {
        return Some(cap.get(2).map_or("", |m| m.as_str()).trim().to_string());
    }

    None
}

pub fn guess_episode(s: &str) -> Result<f64, ScraperError> {
    let patterns = [
        r"(?i)chapter\s+(\d+(\.\d+)?)",
        r"(?i)ch\.\s+(\d+(\.\d+)?)",
        r"(?i)-ch_+(\d+(\.\d+)?)",
        r"第(\d+(\.\d+)?)",
    ];

    let lower_s = s.to_lowercase();

    for pattern in &patterns {
        let re = Regex::new(pattern).expect("Invalid regex pattern");
        if let Some(captured) = re.captures(&lower_s) {
            let number_str = &captured[1];
            return number_str
                .parse()
                .map_err(|_| ScraperError::InvalidChapterNum(number_str.to_owned()));
        }
    }

    s.parse::<f64>()
        .map_err(|_| ScraperError::InvalidChapterNum(s.to_owned()))
}
