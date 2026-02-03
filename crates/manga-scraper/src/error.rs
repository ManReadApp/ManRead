use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use scraper::error::SelectorErrorKind;

#[derive(Debug)]
pub enum InitError {
    InitParseError(String),
    InitParseRegexError(String),
    InitParseReadError(String),
    InvalidHeaderValue,
    NoEngineSelected,
    RequiredFieldMissing,
    InvalidHeaderName,
    InvalidAttrFormat,
    UnknownAttr,
    Toml(toml::de::Error),
    SelectorError(String),
    Reqwest(reqwest::Error),
}

impl From<toml::de::Error> for InitError {
    fn from(err: toml::de::Error) -> Self {
        InitError::Toml(err)
    }
}

impl<'a> From<SelectorErrorKind<'a>> for InitError {
    fn from(value: SelectorErrorKind) -> Self {
        InitError::SelectorError(value.to_string())
    }
}
impl From<InvalidHeaderName> for InitError {
    fn from(value: InvalidHeaderName) -> Self {
        InitError::InvalidHeaderName
    }
}
impl From<InvalidHeaderValue> for InitError {
    fn from(err: InvalidHeaderValue) -> Self {
        InitError::InvalidHeaderValue
    }
}

impl From<reqwest::Error> for InitError {
    fn from(err: reqwest::Error) -> Self {
        InitError::Reqwest(err)
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    NodeNotFound,
    Cloudflare,
    UrlParseError(String),
    Reqwest(reqwest::Error),
}

impl From<reqwest::Error> for ScrapeError {
    fn from(err: reqwest::Error) -> Self {
        ScrapeError::Reqwest(err)
    }
}

impl From<url::ParseError> for ScrapeError {
    fn from(err: url::ParseError) -> Self {
        ScrapeError::UrlParseError(err.to_string())
    }
}

impl From<std::io::Error> for InitError {
    fn from(err: std::io::Error) -> Self {
        InitError::InitParseReadError(err.to_string())
    }
}

impl From<regex::Error> for InitError {
    fn from(err: regex::Error) -> Self {
        InitError::InitParseRegexError(err.to_string())
    }
}

impl From<serde_json::Error> for InitError {
    fn from(err: serde_json::Error) -> Self {
        println!("{err}");
        todo!()
    }
}

pub type Result<T> = std::result::Result<T, InitError>;
