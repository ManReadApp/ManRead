use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::header::COOKIE;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub enum Translator {
    /// https://ichigoreader.notion.site/ccf147c01b3349bfbd6731d3f4ee8b11?v=46cb91d0bfad42b981b20de849f3083f
    Ichigo(Arc<Mutex<IchigoData>>),
    /// https://github.com/zyddnys/manga-image-translator
    MangaImageTranslator,
}

impl Translator {
    pub async fn ichigo(
        username: String,
        password: String,
        uuid: String,
        fingerprint: String,
    ) -> Self {
        Self::Ichigo(Arc::new(Mutex::new(IchigoData {
            username: username.clone(),
            password: password.clone(),
            fingerprint,
            uuid,
            cookie: get_ichigo_cookie(username, password).await.unwrap(),
        })))
    }
}

struct IchigoData {
    username: String,
    password: String,
    fingerprint: String,
    uuid: String,
    cookie: String,
}

#[derive(Serialize)]
struct IchigoRequest {
    fingerprint: String,
    #[serde(rename = "clientUuid")]
    client_uuid: String,
    #[serde(rename = "base64Images")]
    base64images: Vec<String>,
}

impl IchigoRequest {
    fn new(img: PathBuf, fingerprint: String, uuid: String) -> Self {
        let mut bytes = vec![];
        File::open(img).unwrap().read_to_end(&mut bytes).unwrap();
        let base = STANDARD.encode(bytes);
        Self {
            fingerprint,
            client_uuid: uuid,
            base64images: vec![base],
        }
    }
}

pub async fn translate(
    translator: Translator,
    src_target: Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    match translator {
        Translator::Ichigo(v) => ichigo_translate(v, src_target).await,
        Translator::MangaImageTranslator => unimplemented!(),
    }
}

async fn ichigo_translate(
    translator: Arc<Mutex<IchigoData>>,
    src_target: Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    if src_target.is_empty() {
        return Ok(());
    }
    let mut items1 = vec![];
    let mut items2 = vec![];
    let mut count = 0;
    for (src, target) in src_target {
        if count % 2 == 0 {
            items1.push((src, target))
        } else {
            items2.push((src, target))
        }
        count += 1;
    }
    if items2.is_empty() {
        ichigo_translate_instance(translator, items1).await
    } else {
        let v = tokio::join!(
            ichigo_translate_instance(translator.clone(), items1),
            ichigo_translate_instance(translator, items2)
        );
        v.0?;
        v.1
    }
}

async fn ichigo_translate_instance(
    translator: Arc<Mutex<IchigoData>>,
    src_target: Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    let client = Client::new();
    for (src, target) in src_target {
        let rb = {
            let translator = translator.lock().unwrap();
            client
                .post("https://ichigoreader.com/translate")
                .json(&IchigoRequest::new(
                    src,
                    translator.fingerprint.clone(),
                    translator.uuid.clone(),
                ))
                .header(COOKIE, translator.cookie.clone())
        };

        let bytes = download(rb).await?;
        File::create(target).unwrap().write_all(&bytes).unwrap();
    }
    Ok(())
}

async fn download(v: RequestBuilder) -> Result<Vec<u8>, String> {
    for i in 0..5 {
        let data = match v.try_clone().unwrap().send().await {
            Ok(v) => {
                if v.status().is_success() {
                    v.bytes().await.map_err(|v| v.to_string())
                } else {
                    Err(format!("Statuscode: {}", v.status().to_string()))
                }
            }
            Err(v) => Err(v.to_string()),
        };
        if let Ok(v) = data {
            return Ok(v.to_vec());
        }
        if i == 4 {
            return data.map(|v| v.to_vec());
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ichigo() {
        let translator = Translator::Ichigo(Arc::new(Mutex::new(IchigoData {
            username: "".to_string(),
            password: "".to_string(),
            fingerprint: "1".to_string(),
            uuid: "1".to_string(),
            cookie: "".to_string(),
        })));
        translate(
            translator,
            vec![(
                "./Wikipe-tan_manga_page1.jpg".into(),
                "./Wikipe-tan_manga_page1.json".into(),
            )],
        )
        .await
        .unwrap();
    }
}

async fn get_ichigo_cookie(username: String, password: String) -> Result<String, String> {
    let v = download(
        Client::new()
            .post("https://ichigoreader.com/auth/login")
            .json(&IchigoLogin::new(username, password)),
    )
    .await?;
    let v: Value = serde_json::from_slice(v.as_slice()).map_err(|v| v.to_string())?;
    let v: Tokens = serde_json::from_value(v["tokens"].clone()).map_err(|v| v.to_string())?;
    Ok(format!(
        "access_cookie={}; refresh_token_cookie={}",
        v.access_token, v.refresh_token
    ))
}

#[derive(Serialize)]
struct IchigoLogin {
    email: String,
    password: String,
}

impl IchigoLogin {
    fn new(username: String, password: String) -> Self {
        Self {
            email: username,
            password,
        }
    }
}

#[derive(Deserialize)]
struct Tokens {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}
