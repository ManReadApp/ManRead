use std::{
    io::{self, BufReader},
    sync::Arc,
};

use futures_util::{StreamExt as _, TryStreamExt as _};
use image::ImageReader;
use tokio::{fs::File, sync::Semaphore};
use tokio_util::io::ReaderStream;

use crate::{
    backends::{ByteStream, StorageWriter},
    error::ProcessingError,
    temp::{MemoryTempData, TempData},
};

pub(crate) struct PreparedUpload {
    pub(crate) handle: String,
    pub(crate) dims: Option<(u32, u32)>,
    pub(crate) ext: Option<(&'static str, &'static str)>,
}

#[async_trait::async_trait]
pub(crate) trait MediaWorker: Send + Sync {
    async fn detect_ext(&self, source: &Arc<dyn TempData>) -> Option<(&'static str, &'static str)>;
    async fn process_and_upload(
        &self,
        source: Arc<dyn TempData>,
        writer: Arc<dyn StorageWriter + Send + Sync>,
        transcode_sem: Arc<Semaphore>,
    ) -> Result<PreparedUpload, ProcessingError>;
}

pub(crate) struct DefaultMediaWorker;

pub(crate) fn file_to_bytestream(file: File) -> ByteStream {
    ReaderStream::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed()
}

#[async_trait::async_trait]
impl MediaWorker for DefaultMediaWorker {
    async fn detect_ext(&self, source: &Arc<dyn TempData>) -> Option<(&'static str, &'static str)> {
        let bytes = source.read_head(8192).await.ok()?;
        let kind = infer::get(&bytes)?;
        let ext = kind.extension();
        if ext == "jpg" {
            Some((kind.mime_type(), "jpeg"))
        } else {
            Some((kind.mime_type(), kind.extension()))
        }
    }

    async fn process_and_upload(
        &self,
        source: Arc<dyn TempData>,
        writer: Arc<dyn StorageWriter + Send + Sync>,
        transcode_sem: Arc<Semaphore>,
    ) -> Result<PreparedUpload, ProcessingError> {
        let _permit = transcode_sem
            .acquire()
            .await
            .map_err(|_| ProcessingError::SemaphoreClosed)?;

        let ext = self.detect_ext(&source).await;
        let is_image = ext.map(|(m, _)| m.starts_with("image/")).unwrap_or(false);
        let allowed = matches!(
            ext.map(|(_, e)| e),
            Some("avif" | "webp" | "png" | "jpeg" | "jpg" | "gif")
        );

        let mut final_ext = ext;
        let mut dims = None;
        let upload_handle = format!("temp/{}", uuid::Uuid::new_v4());
        if is_image {
            //TODO: spawn hashing task, which will run in a worker pool. States are, waiting, done-processing, processing. Items that are truly done or had an error get removed. its possible to attach an id to them or mark them as canceled in any state. marked as canceled will get removed & not further processed. if the id is set it will flush the hash to the db. it is only used at done-processing. wake up on done-processing and the others should just set the id, because it will reach done processing anyway and then flush it to the db.
        }

        if is_image && !allowed {
            let buffer = source
                .read_all()
                .await
                .map_err(ProcessingError::ReadTempFile)?;

            let (jpeg_bytes, size) = tokio::task::spawn_blocking(move || {
                let img = image::load_from_memory(&buffer)?;
                let mut out = Vec::new();
                let size = (img.width(), img.height());
                let rgb = img.to_rgb8();
                let mut enc = image::codecs::jpeg::JpegEncoder::new(&mut out);
                enc.encode(
                    &rgb,
                    rgb.width(),
                    rgb.height(),
                    image::ColorType::Rgb8.into(),
                )?;
                Ok::<_, image::ImageError>((out, size))
            })
            .await
            .map_err(ProcessingError::ImageWorkerJoin)?
            .map_err(ProcessingError::ImageConversion)?;

            dims = Some(size);
            final_ext = Some(("image/jpeg", "jpeg"));

            let converted: Arc<dyn TempData> = Arc::new(MemoryTempData::from_bytes(jpeg_bytes));
            writer
                .write(
                    &upload_handle,
                    converted
                        .open_stream()
                        .await
                        .map_err(ProcessingError::UploadConverted)?,
                )
                .await
                .map_err(ProcessingError::UploadConverted)?;
        } else {
            if is_image {
                let s = source.clone();
                let s = s.open().await.unwrap();
                dims = Some(
                    tokio::task::spawn_blocking(move || {
                        let v: ImageReader<BufReader<_>> = ImageReader::new(BufReader::new(s));
                        v.with_guessed_format()
                            .map_err(|e| image::ImageError::IoError(e))
                            .and_then(|v| v.into_dimensions())
                    })
                    .await
                    .map_err(ProcessingError::DimensionWorkerJoin)?
                    .map_err(ProcessingError::ReadImageDimensions)?,
                );
            }
            writer
                .write(
                    &upload_handle,
                    source
                        .open_stream()
                        .await
                        .map_err(ProcessingError::OpenTempFileForUpload)?,
                )
                .await
                .map_err(ProcessingError::UploadTemp)?;
        }

        Ok(PreparedUpload {
            handle: upload_handle,
            dims,
            ext: final_ext,
        })
    }
}
