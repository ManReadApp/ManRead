use std::collections::HashMap;
#[cfg(feature = "curl")]
use crate::error::status_code_err;
use crate::ScrapeError;
#[cfg(feature = "curl")]
use curl::easy::List;
#[cfg(feature = "curl")]
use reqwest::Method;
use reqwest::RequestBuilder;
#[cfg(feature = "curl")]
use std::sync::{Arc, Mutex};
use reqwest::header::COOKIE;

pub async fn download(v: RequestBuilder, cloudflare: bool) -> Result<String, ScrapeError> {
    let data = download_(v.try_clone().unwrap()).await;
    if cloudflare {
        if let Ok(data)  = &data {
            if data.contains("<title>Just a moment...</title>") {
                let (client, req) = v.try_clone().unwrap().build_split();
                let url = req.unwrap().url().to_string();
                let cookies: HashMap<String, String> = client.get(format!("http://127.0.0.1:8000/cookie?url={}", urlencoding::encode(url.as_str()))).send().await?.json().await?;
                let cookies = cookies.iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<String>>()
                    .join("; ");
                return download_(v.header(COOKIE, cookies)).await;
            }
        }
    }

    data
}
pub async fn download_(v: RequestBuilder) -> Result<String, ScrapeError> {
    for i in 0..5 {
        #[cfg(feature = "curl")]
        let data = {
            let data = v.try_clone().unwrap().build().unwrap();
            let mut buf = Arc::new(Mutex::new(Vec::new()));
            let mut handle = curl::easy::Easy::new();
            handle.url(data.url().as_str())?;
            let b = buf.clone();
            handle.write_function(move |data| {
                b.lock().unwrap().extend_from_slice(data);
                Ok(data.len())
            })?;
            if data.method() == Method::POST {
                handle.post(true)?;
            }
            let mut list = List::new();
            for data in data.headers() {
                list.append(&format!("{}: {}", data.0, data.1.to_str().unwrap()))?;
            }
            handle.http_headers(list)?;
            handle.perform()?;

            let res;
            loop {
                let resp = handle.response_code()?;
                if resp == 0 || buf.lock().unwrap().is_empty() {
                    warn!("request is not blocking, code needs to be fixed");
                    continue;
                }
                res = if resp >= 200 && resp < 300 {
                    Ok(String::from_utf8(buf.lock().unwrap().to_vec()).unwrap())
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
        let data = match v.try_clone().unwrap().send().await {
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
