#[cfg(feature = "curl")]
use crate::error::status_code_err;
use crate::ScrapeError;
#[cfg(feature = "curl")]
use curl::easy::List;
#[cfg(feature = "curl")]
use log::warn;
use reqwest::header::{COOKIE, USER_AGENT};
#[cfg(feature = "curl")]
use reqwest::Method;
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::collections::HashMap;
#[cfg(feature = "curl")]
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct CfBypassResponse {
    cookies: HashMap<String, String>,
    user_agent: String,
}

pub async fn download(v: RequestBuilder, cloudflare: bool) -> Result<String, ScrapeError> {
    let data = download_(
        v.try_clone()
            .ok_or(ScrapeError::invalid_request(format!("{:?} is invalid", v)))?,
    )
    .await;
    if cloudflare {
        if let Ok(data) = &data {
            if data.contains("<title>Just a moment...</title>") {
                let (client, req) = v
                    .try_clone()
                    .ok_or(ScrapeError::invalid_request(format!("{:?} is invalid", v)))?
                    .build_split();
                let url = req?.url().to_string();
                let headers: CfBypassResponse = client
                    .get(format!(
                        "http://127.0.0.1:8000/cookies?url={}",
                        urlencoding::encode(url.as_str())
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;
                let (client, req) = v.build_split();
                let req = req?;
                let req = client
                    .get(req.url().to_string())
                    .header(
                        COOKIE,
                        format!(
                            "cf_clearance={}",
                            headers
                                .cookies
                                .get("cf_clearance")
                                .ok_or(ScrapeError::node_not_found())
                                .to_string()
                        ),
                    )
                    .header(USER_AGENT, headers.user_agent);
                return download_(req).await;
            }
        }
    }

    data
}
pub async fn download_(v: RequestBuilder) -> Result<String, ScrapeError> {
    for i in 0..5 {
        #[cfg(feature = "curl")]
        let data = {
            let data = v
                .try_clone()
                .ok_or(ScrapeError::invalid_request(format!("{:?} is invalid", v)))?
                .build()?;
            let mut buf = Arc::new(Mutex::new(Vec::new()));
            let mut handle = curl::easy::Easy::new();
            handle.url(data.url().as_str())?;
            let b = buf.clone();
            handle.write_function(move |data| {
                b.lock()?.extend_from_slice(data);
                Ok(data.len())
            })?;
            if data.method() == Method::POST {
                handle.post(true)?;
            }
            let mut list = List::new();
            for data in data.headers() {
                list.append(&format!(
                    "{}: {}",
                    data.0,
                    data.1.to_str().unwrap_or_default()
                ))?;
            }
            handle.http_headers(list)?;
            handle.perform()?;

            let res;
            loop {
                let resp = handle.response_code()?;
                if resp == 0 || buf.lock()?.is_empty() {
                    warn!("request is not blocking, code needs to be fixed");
                    continue;
                }
                res = if resp >= 200 && resp < 300 {
                    Ok(String::from_utf8(buf.lock()?.to_vec())?)
                } else {
                    Err(status_code_err(resp))
                };
                break;
            }
            res
        };
        #[cfg(feature = "curl")]
        if let Ok(data) = data {
            return Ok(data);
        }

        #[cfg(feature = "curl")]
        if i == 4 {
            return data;
        }
        #[cfg(not(feature = "curl"))]
        let data = match v
            .try_clone()
            .ok_or(ScrapeError::invalid_request(format!("{:?} is invalid", v)))?
            .send()
            .await
        {
            Ok(v) => v.text().await,
            Err(v) => Err(v),
        };
        #[cfg(not(feature = "curl"))]
        if let Ok(v) = data {
            return Ok(v);
        }
        #[cfg(not(feature = "curl"))]
        if i == 4 {
            return Ok(data?);
        }
    }
    unreachable!()
}
