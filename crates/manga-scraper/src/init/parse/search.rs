use std::{fs::read_to_string, path::Path, sync::Arc};

use regex::Regex;
use scraper::Selector;
use scraper_module::SearchScraper;
use serde::Deserialize;

use crate::{init::scraper::Engine, processors::PostSearchProcessor, InitError};

use super::metadata::parse_engine;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Info {
    url: Option<String>,
    cover: Option<String>,
    a: Option<String>,
    selector: Option<String>,
    label: Option<String>,
    #[serde(rename = "type")]
    ty: Option<String>,
    status: Option<String>,
    pages: Option<String>,
    pages_attr: Option<String>,
    pages_regex: Option<String>,
    single_page: Option<bool>,
    next_only: Option<bool>,
}

impl Info {
    pub fn fields() -> Vec<&'static str> {
        vec![
            "url",
            "cover",
            "a",
            "selector",
            "label",
            "ty",
            "status",
            "pages",
            "pages_attr",
            "pages_regex",
            "single_page",
            "next_only",
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct Search {
    general: Option<Info>,
    no_query: Option<Info>,
    query: Option<Info>,
}

#[derive(Debug)]
pub struct InfoSelectors {
    pub url: String,
    pub pages: Option<Selector>,
    pub pages_attr: Option<String>,
    pub pages_regex: Option<Regex>,
    pub single_page: Option<bool>,
    pub next_only: Option<bool>,
    pub cover: Option<Selector>,
    pub a: Option<Selector>,
    pub selector: Selector,
    pub label: Option<Selector>,
    pub ty: Option<Selector>,
    pub status: Option<Selector>,
}

pub struct InterpretedSearch {
    pub engine: Engine,
    pub no_query: InfoSelectors,
    pub query: InfoSelectors,
    pub post_search: Vec<(Arc<dyn PostSearchProcessor + Send + Sync>, String)>,
}

pub fn parse(
    file: &Path,
    processors: Vec<&str>,
    post_search: Vec<(Arc<dyn PostSearchProcessor + Send + Sync>, String)>,
) -> crate::Result<Arc<dyn SearchScraper>> {
    Ok(Arc::new(InterpretedSearch::new(
        file,
        processors,
        post_search,
    )?))
}

impl InterpretedSearch {
    pub fn new(
        file: &Path,
        processors: Vec<&str>,
        post_search: Vec<(Arc<dyn PostSearchProcessor + Send + Sync>, String)>,
    ) -> crate::Result<Self> {
        let content = read_to_string(file)?;
        let data: Search = toml::from_str(&content)?;

        let query_data = selectors(merge(data.query, data.general.clone()))?;

        let no_query_data = selectors(merge(data.no_query, data.general))?;

        Ok(Self {
            no_query: no_query_data,
            query: query_data,
            engine: parse_engine(processors)?,
            post_search,
        })
    }
}

fn selectors(info: Info) -> crate::Result<InfoSelectors> {
    Ok(InfoSelectors {
        url: info.url.ok_or(InitError::RequiredFieldMissing)?,
        cover: info
            .cover
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        a: info
            .a
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        selector: Selector::parse(&info.selector.ok_or(InitError::RequiredFieldMissing)?)?,
        label: info
            .label
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        ty: info
            .ty
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        status: info
            .status
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        pages: info
            .pages
            .map(|v| Selector::parse(&v).map_err(InitError::from))
            .transpose()?,
        pages_attr: info.pages_attr,
        single_page: info.single_page,
        next_only: info.next_only,
        pages_regex: info
            .pages_regex
            .map(|v| Regex::new(&v).map_err(InitError::from))
            .transpose()?,
    })
}

fn merge(info: Option<Info>, general: Option<Info>) -> Info {
    match (info, general) {
        (Some(info), None) => info,
        (None, Some(info)) => info,
        (Some(mut info), Some(general)) => {
            macro_rules! merge_fields {
                ($target:expr, $source:expr, [$( $field:ident ),* $(,)?]) => {
                    $(
                        if $target.$field.is_none() {
                            $target.$field = $source.$field.clone();
                        }
                    )*
                };
            }

            let items = Info::fields()
                .into_iter()
                .filter(|v| {
                    ![
                        "url",
                        "cover",
                        "a",
                        "selector",
                        "label",
                        "ty",
                        "status",
                        "pages",
                        "pages_attr",
                        "pages_regex",
                        "single_page",
                        "next_only",
                    ]
                    .contains(&v)
                })
                .collect::<Vec<_>>();
            if !items.is_empty() {
                panic!("missing fields: {:?}", items)
            }
            merge_fields!(
                info,
                general,
                [
                    url,
                    cover,
                    a,
                    selector,
                    label,
                    ty,
                    status,
                    pages,
                    pages_attr,
                    pages_regex,
                    single_page,
                    next_only
                ]
            );
            info
        }
        _ => Info::default(),
    }
}
