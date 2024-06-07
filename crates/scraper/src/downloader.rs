use reqwest::{Error, RequestBuilder};

pub async fn download(v: RequestBuilder) -> Result<String, Error> {
    for i in 0..5 {
        let data = match v.try_clone().unwrap().send().await {
            Ok(v) => v.text().await,
            Err(v) => Err(v),
        };
        if let Ok(v) = data {
            return Ok(v);
        }
        if i == 4 {
            return data;
        }
    }
    unreachable!()
}
