use std::{collections::HashMap, str::FromStr, sync::Arc};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, COOKIE, REFERER, USER_AGENT},
    ClientBuilder, IntoUrl,
};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{InitError, ScrapeError};

pub struct Engine {
    kind: EngineKind,
    headers: HeaderMap,
}

pub enum EngineMode {
    ReqwestNativeTls,
    ReqwestRustlsTls,
    Curl,
}
impl Engine {
    pub fn new(reqwest: EngineMode, headers: HashMap<String, String>) -> Result<Self, InitError> {
        let mut map = HeaderMap::new();
        map.append(
            USER_AGENT,
            HeaderValue::from_str(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X x.y; rv:42.0) Gecko/20100101 Firefox/42.0",
            )?,
        );
        for (k, v) in headers {
            map.append(HeaderName::from_str(&k)?, HeaderValue::from_str(&v)?);
        }

        Ok(Self {
            kind: match reqwest {
                EngineMode::ReqwestNativeTls => EngineKind::Reqwest(
                    ClientBuilder::new()
                        .danger_accept_invalid_certs(true)
                        .use_native_tls()
                        .build()?,
                ),
                EngineMode::ReqwestRustlsTls => EngineKind::Reqwest(
                    ClientBuilder::new()
                        .danger_accept_invalid_certs(true)
                        .use_rustls_tls()
                        .build()?,
                ),
                EngineMode::Curl => todo!(),
            },
            headers: map,
        })
    }
}
pub enum EngineKind {
    Reqwest(reqwest::Client),
}

static SHARED_COOKIES: Lazy<Arc<DashMap<String, String>>> = Lazy::new(|| Arc::new(DashMap::new()));
static SHARED_USER_AGENTS: Lazy<Arc<DashMap<String, String>>> =
    Lazy::new(|| Arc::new(DashMap::new()));

pub fn set_cookie(url: &Url, cookie: String) {
    if let Some(domain) = url.host_str() {
        SHARED_COOKIES.insert(domain.to_string(), cookie);
    }
}

/// Get a cookie for the domain of the given URL
pub fn get_cookie(url: impl IntoUrl) -> Option<String> {
    let url = url.into_url().ok()?;
    url.host_str()
        .and_then(|domain| SHARED_COOKIES.get(domain).map(|c| c.value().clone()))
}

/// Set a user agent for the domain of the given URL
pub fn set_user_agent(url: &Url, user_agent: String) {
    if let Some(domain) = url.host_str() {
        SHARED_USER_AGENTS.insert(domain.to_string(), user_agent);
    }
}

/// Get a user agent for the domain of the given URL
pub fn get_user_agent(url: impl IntoUrl) -> Option<String> {
    let url = url.into_url().ok()?;
    url.host_str()
        .and_then(|domain| SHARED_USER_AGENTS.get(domain).map(|ua| ua.value().clone()))
}

#[derive(Deserialize)]
struct CloudflareBypass {
    user_agent: String,
    cookies: HashMap<String, String>,
}
fn get_site_root_referer(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let scheme = url.scheme();
    let host = url.host_str()?;
    let port = url.port().map(|p| format!(":{}", p)).unwrap_or_default();

    Some(format!("{}://{}{}", scheme, host, port))
}

impl Engine {
    pub async fn request_str(&self, get: bool, url: &str) -> Result<String, ScrapeError> {
        Ok(String::from_utf8_lossy(&self.request(get, url).await?)
            .replace("<noscript>", "")
            .replace("</noscript>", ""))
    }
    pub async fn request(&self, get: bool, url: &str) -> Result<Vec<u8>, ScrapeError> {
        let mut err = Ok(vec![]);
        for _ in 0..3 {
            match self.request_single(get, url).await {
                Ok(text) => {
                    let cloudflare = b"<title>Just a moment...</title>";
                    if text
                        .windows(cloudflare.len())
                        .any(|window| window == cloudflare)
                    {
                        err = Err(ScrapeError::Cloudflare);
                        match &self.kind {
                            EngineKind::Reqwest(client) => {
                                let url = Url::parse(url).unwrap();
                                if let Ok(v) = client
                                    .post("http://127.0.0.1:8000/bypass-cloudflare")
                                    .json(&json!({"url": url.to_string(), "refresh": false}))
                                    .send()
                                    .await
                                {
                                    if let Ok(v) = v.json::<CloudflareBypass>().await {
                                        set_user_agent(&url, v.user_agent);
                                        set_cookie(
                                            &url,
                                            v.cookies
                                                .into_iter()
                                                .map(|(k, v)| format!("{}={}", k, v))
                                                .collect::<Vec<_>>()
                                                .join("; "),
                                        );
                                    }
                                }
                            }
                        };
                    } else {
                        return Ok(text);
                    }
                }
                Err(e) => {
                    err = Err(ScrapeError::Reqwest(e));
                }
            }
        }
        return err;
    }

    async fn request_single(&self, get: bool, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        let mut headers = self.headers.clone();
        if !headers.contains_key(REFERER) {
            if let Some(v) = get_site_root_referer(url) {
                headers.append(REFERER, HeaderValue::from_str(&v).unwrap());
            }
        }
        if let Some(user_agent) = &get_user_agent(url) {
            headers.remove(USER_AGENT);

            headers.append(USER_AGENT, HeaderValue::from_str(user_agent).unwrap());
        }

        if let Some(cokies) = &get_cookie(url) {
            headers.remove(COOKIE);
            headers.append(COOKIE, HeaderValue::from_str(cokies).unwrap());
        }

        match &self.kind {
            EngineKind::Reqwest(client) => {
                if get {
                    client
                        .get(url)
                        .headers(headers)
                        .send()
                        .await?
                        .bytes()
                        .await
                        .map(|v| v.to_vec())
                } else {
                    client
                        .post(url)
                        .headers(headers)
                        .send()
                        .await?
                        .bytes()
                        .await
                        .map(|v| v.to_vec())
                }
            }
        }
    }
}
