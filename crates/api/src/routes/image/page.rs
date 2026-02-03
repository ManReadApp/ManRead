use std::io;

use actix_web::{
    web::{Bytes, Data, Json},
    HttpResponse,
};
use actix_web_grants::AuthorityGuard;
use api_structure::{models::auth::role::Permission, req::reader::image::MangaReaderImageRequest};
use apistos::api_operation;
use futures::stream::once;
use futures::StreamExt;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{error::ApiResult, Config};

use super::page_translation::get_translation;

#[api_operation(
    tag = "image",
    summary = "Gets the page & translation of a manga",
    description = r###""###
)]
pub(crate) async fn exec(
    Json(data): Json<MangaReaderImageRequest>,
    config: Data<Config>,
) -> ApiResult<HttpResponse> {
    let translation = serde_json::to_string(&get_translation(&config, &data)?)?;
    let path = config
        .root_folder
        .join("mangas")
        .join(data.manga_id)
        .join(data.chapter_id)
        .join(data.version_id)
        .join(format!("{}.{}", data.page, data.file_ext));
    let file_size = path.metadata()?.len() as u64;
    let translation_length = translation.len() as u64;

    let file_size_bytes = file_size.to_be_bytes();
    let translation_length_bytes = translation_length.to_be_bytes();
    let file = File::open(path).await.unwrap();

    let file_stream = ReaderStream::new(file);
    let translation_stream =
        once(async move { Ok::<_, io::Error>(Bytes::from(translation.into_bytes())) });

    let stream = once(async move { Ok::<_, io::Error>(Bytes::from(file_size_bytes.to_vec())) })
        .chain(file_stream.map(|chunk| chunk.map(Bytes::from)))
        .chain(once(async move {
            Ok::<_, io::Error>(Bytes::from(translation_length_bytes.to_vec()))
        }))
        .chain(translation_stream);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .streaming(stream))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/page_t").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
