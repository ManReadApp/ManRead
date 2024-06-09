use crate::get_app_data;
use crate::window_storage::Page;
use api_structure::error::{ApiErr, ApiErrorType, ClientError};
use api_structure::Request;
use egui::mutex::Mutex;
use egui::{Context, Label, Sense, Ui};
use ethread::ThreadHandler;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use reqwest::multipart::Part;
use reqwest::{Body, Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::VecDeque;
use std::mem;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::fs::File;

pub struct Fetcher<T: DeserializeOwned = bool> {
    request: Request,
    response: Response<T>,
    context: Arc<Mutex<Option<Context>>>,
}

impl<T: DeserializeOwned + Send> Fetcher<T> {
    /// returns true if fetcher is loading
    pub fn loading(&self) -> bool {
        self.response.loading()
    }
    /// new
    pub fn new(request: Request) -> Self {
        Self {
            request,
            response: Response::None,
            context: Arc::new(Default::default()),
        }
    }

    /// new
    pub fn new_ctx(request: Request, ctx: Context) -> Self {
        Self {
            request,
            response: Response::None,
            context: Arc::new(Mutex::new(Some(ctx))),
        }
    }

    /// sets body which is send
    pub fn set_body(&mut self, data: impl Serialize) {
        let s = serde_json::to_string(&data)
            .expect("Should always be able to convert struct to string");
        self.request.set_content(s);
    }

    /// sets context for request reload
    pub fn set_ctx(&mut self, ctx: Context) {
        self.context.lock().replace(ctx);
    }

    /// sends request & sets Response 2 processing
    pub fn send(&mut self) {
        let request = get_request_builder(&get_app_data().client, &self.request);
        let shared: Arc<Mutex<VecDeque<Vec<u8>>>> = Arc::new(Default::default());
        let sharedc = shared.clone();

        let context = self.context.clone();
        let auth = self.request.auth;
        let v = self.context.as_ref().lock().clone();
        let ctx = match &v {
            None => None,
            Some(v) => Some(v),
        };
        let th: ThreadHandler<Result<Vec<u8>, Errors>> = ThreadHandler::new_async_ctx(
            async move {
                let request = if auth {
                    add_auth(request).await
                } else {
                    request
                };
                let resp = request.send().await.map_err(|e| {
                    Errors::ClientErr(ClientError {
                        message: e.to_string(),
                        cause: None,
                        data: None,
                    })
                })?;
                let bytes = false;
                if bytes {
                    let mut stream = resp.bytes_stream();
                    while let Some(item) = stream.next().await {
                        let item: Vec<u8> = item
                            .map_err(|e| {
                                Errors::ClientErr(ClientError {
                                    message: "Failed to get chunk".to_string(),
                                    cause: Some(e.to_string()),
                                    data: None,
                                })
                            })
                            .map(|v| v.to_vec())?;
                        sharedc.lock().push_back(item);
                    }
                    context.lock().as_ref().unwrap().request_repaint();
                    Ok(vec![])
                } else {
                    let suc = resp.status().is_success();
                    let byts = resp.bytes().await.map_err(|e| {
                        Errors::ClientErr(ClientError {
                            message: e.to_string(),
                            cause: None,
                            data: None,
                        })
                    })?;
                    if suc {
                        context.lock().as_ref().unwrap().request_repaint();
                        Ok(byts.to_vec())
                    } else {
                        let err =
                            serde_json::from_slice::<ApiErr>(&byts)
                                .ok()
                                .unwrap_or_else(|| ApiErr {
                                    message: String::from_utf8(byts.to_vec()).ok(),
                                    cause: None,
                                    err_type: ApiErrorType::InternalError,
                                });
                        Err(Errors::ApiErr(err))
                    }
                }
            },
            ctx,
        );
        self.response = Response::Procressing {
            processed: vec![],
            shared,
            th,
        };
    }

    ///moves bytes out of the chunks and puts them into a vec
    fn process_data(&mut self) -> bool {
        let mut done = true;
        if let Response::Procressing {
            processed,
            th,
            shared,
        } = &mut self.response
        {
            while let Some(mut bytes) = shared.lock().pop_front() {
                processed.append(&mut bytes);
            }

            if let Some(data) = th.task.ready() {
                let complete = match data {
                    Ok(r) => {
                        if self.request.bytes {
                            let mut res = vec![];
                            mem::swap(&mut res, processed);
                            Complete::Bytes(res)
                        } else {
                            Complete::Json(serde_json::from_slice(r).unwrap())
                        }
                    }
                    Err(v) => match v {
                        Errors::ClientErr(e) => Complete::Error(e.clone()),
                        Errors::ApiErr(e) => Complete::ApiError(e.clone()),
                    },
                };

                self.response = Response::Done(complete);
            } else {
                done = false;
            }
        }

        done
    }

    /// gets the result
    /// sends if not send before
    pub fn result(&mut self) -> Option<&Complete<T>> {
        if matches!(self.response, Response::None) {
            None
        } else if self.response.loading() {
            let done = self.process_data();
            if done {
                self.result()
            } else {
                None
            }
        } else if let Response::Done(v) = &self.response {
            Some(v)
        } else {
            unreachable!()
        }
    }

    pub fn take_result(mut self) -> Option<Complete<T>> {
        if matches!(self.response, Response::None) {
            None
        } else if self.response.loading() {
            let done = self.process_data();
            if done {
                self.take_result()
            } else {
                None
            }
        } else if let Response::Done(v) = self.response {
            Some(v)
        } else {
            unreachable!()
        }
    }
}

enum Response<T> {
    None,
    Procressing {
        processed: Vec<u8>,
        shared: Arc<Mutex<VecDeque<Vec<u8>>>>,
        th: ThreadHandler<Result<Vec<u8>, Errors>>,
    },
    Done(Complete<T>),
}

impl<T> Response<T> {
    fn loading(&self) -> bool {
        matches!(self, Response::Procressing { .. })
    }
}

#[derive(Clone)]
pub enum Complete<T> {
    ApiError(ApiErr),
    Error(ClientError),
    Bytes(Vec<u8>),
    Json(T),
}

impl<T> Complete<T> {
    pub fn display_error(&self, ui: &mut Ui) {
        match self {
            Complete::ApiError(e) => {
                ui.add(
                    Label::new(e.message.clone().unwrap_or("Unknown Error".to_string()))
                        .sense(Sense::click()),
                )
                .on_hover_text(format!(
                    "{}: {}",
                    e.err_type,
                    e.cause.clone().unwrap_or_default()
                ));
            }
            Complete::Error(e) => {
                ui.add(Label::new(e.message.clone()).sense(Sense::click()))
                    .on_hover_text(format!(
                        "{}: {}",
                        e.cause.clone().unwrap_or_default(),
                        e.data.clone().unwrap_or_default()
                    ));
            }
            _ => {}
        }
    }
}

fn get_request_builder(client: &Client, r: &Request) -> RequestBuilder {
    let hm = HeaderMap::from_iter(r.headers.iter().map(|(a, b)| {
        (
            HeaderName::from_str(a.as_str()).unwrap(),
            HeaderValue::from_str(b.as_str()).unwrap(),
        )
    }));
    client
        .request(Method::from_str(&r.method).unwrap(), r.url.clone())
        .headers(hm)
        .body(r.req_body.clone())
}

async fn add_auth(r: RequestBuilder) -> RequestBuilder {
    let app = get_app_data();
    let user = app.get_access_token().await;
    if let Some(access_token) = user {
        r.header(AUTHORIZATION, format!("Bearer {}", access_token))
    } else {
        app.change(Page::SignIn, Page::all());
        r
    }
}
pub enum UploadFile {
    #[cfg(not(target_arch = "wasm32"))]
    Path(String),
    Bytes(Vec<u8>),
}

impl From<Vec<u8>> for UploadFile {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<String> for UploadFile {
    fn from(value: String) -> Self {
        Self::Path(value)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<PathBuf> for UploadFile {
    fn from(value: PathBuf) -> Self {
        Self::Path(value.to_str().unwrap_or_default().to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<&Path> for UploadFile {
    fn from(value: &Path) -> Self {
        Self::Path(value.to_str().unwrap_or_default().to_string())
    }
}

pub async fn upload_image(
    ctx: Context,
    data: UploadFile,
    name: Option<String>,
) -> Option<Vec<(String, String)>> {
    let form = reqwest::multipart::Form::new();
    let body = match data {
        #[cfg(not(target_arch = "wasm32"))]
        UploadFile::Path(f) => Body::from(File::open(f).await.unwrap()),
        UploadFile::Bytes(v) => Body::from(v),
    };
    let part = Part::stream(body).file_name(name.unwrap_or("file.png".to_string()));
    let form = form.part("image[]", part);
    let base = &get_app_data().url;
    let builder = get_app_data()
        .client
        .post(base.join("upload_images").unwrap())
        .multipart(form);
    let v = builder.send().await.ok()?;
    let v = v.json().await.ok();
    ctx.request_repaint();
    v
}

#[allow(dead_code)]
pub async fn upload_images(
    ctx: Context,
    data: Vec<(String, UploadFile)>,
) -> Option<Vec<(String, String)>> {
    let mut form = reqwest::multipart::Form::new();
    for (name, data) in data {
        let body = match data {
            #[cfg(not(target_arch = "wasm32"))]
            UploadFile::Path(f) => Body::from(File::open(f).await.unwrap()),
            UploadFile::Bytes(v) => Body::from(v),
        };
        let part = Part::stream(body).file_name(name);
        form = form.part("image[]", part);
    }

    let base = &get_app_data().url;
    let builder = get_app_data()
        .client
        .post(base.join("upload_images").unwrap())
        .multipart(form);
    let v = builder.send().await.ok()?;
    let v = v.json().await.ok();
    ctx.request_repaint();
    v
}

enum Errors {
    ClientErr(ClientError),
    ApiErr(ApiErr),
}
