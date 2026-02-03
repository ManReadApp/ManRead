use std::{collections::HashMap, str::FromStr as _};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Method,
};

use scraper_module::{
    req::{ClientT, MyValue, RequestBuilder, RequestBuilderT, Response, ResponseT},
    ScraperError,
};
use serde_json::Value;

pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
                .build()
                .unwrap(),
        }
    }
}

impl ClientT for ReqwestClient {
    fn method(&self, method: &str, url: &str) -> RequestBuilder {
        RequestBuilder {
            r: Box::new(ReqwestRequestBuilder {
                req: Some(self.client.request(Method::from_str(method).unwrap(), url)),
            }),
        }
    }
}

pub struct ReqwestResponse {
    resp: reqwest::Response,
}

#[async_trait]
impl ResponseT for ReqwestResponse {
    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, ScraperError> {
        Ok(self.resp.bytes().await?.to_vec())
    }

    fn headers(&self) -> Vec<(&str, &str)> {
        self.resp
            .headers()
            .into_iter()
            .map(|(h, v)| (h.as_str(), v.to_str().unwrap()))
            .collect()
    }

    fn status(&self) -> u16 {
        self.resp.status().as_u16()
    }
}

pub struct ReqwestRequestBuilder {
    req: Option<reqwest::RequestBuilder>,
}

#[async_trait]
impl RequestBuilderT for ReqwestRequestBuilder {
    fn set_header(&mut self, key: &str, value: &str) {
        self.req = Some(self.req.take().unwrap().header(key, value));
    }

    fn set_headers(&mut self, headers: HashMap<String, Vec<u8>>) {
        self.req = Some(
            self.req.take().unwrap().headers(
                headers
                    .into_iter()
                    .map(|v| {
                        (
                            HeaderName::from_str(&v.0).unwrap(),
                            HeaderValue::from_bytes(&v.1).unwrap(),
                        )
                    })
                    .collect(),
            ),
        );
    }

    fn set_body(&mut self, body: String) {
        self.req = Some(self.req.take().unwrap().body(body));
    }

    fn set_basic_auth(&mut self, username: &str, password: Option<&str>) {
        self.req = Some(self.req.take().unwrap().basic_auth(username, password));
    }

    fn set_bearer_auth(&mut self, token: &str) {
        self.req = Some(self.req.take().unwrap().bearer_auth(token));
    }

    fn set_form(&mut self, form: &HashMap<String, String>) {
        self.req = Some(self.req.take().unwrap().form(form));
    }

    fn set_json(&mut self, json: MyValue) {
        self.req = Some(self.req.take().unwrap().json(&Value::from(json)));
    }

    async fn send(mut self: Box<Self>) -> Result<Response, ScraperError> {
        Ok(Response {
            r: Box::new(
                self.req
                    .take()
                    .unwrap()
                    .send()
                    .await
                    .map(|v| ReqwestResponse { resp: v })?,
            ),
        })
    }
}
