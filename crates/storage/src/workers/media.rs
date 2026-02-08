use std::{io::Cursor, path::Path, sync::Arc};

use async_tempfile::TempFile;
use futures_util::{StreamExt as _, TryStreamExt as _};
use tokio::{
    fs::File,
    io::{self, AsyncSeekExt as _, AsyncWriteExt as _},
    sync::Semaphore,
};
use tokio_util::io::ReaderStream;

use crate::{
    backends::{ByteStream, StorageWriter},
    error::ProcessingError,
    temp::{FileTempData, MemoryTempData, TempData},
};

pub(crate) struct PreparedUpload {
    pub(crate) handle: String,
    pub(crate) dims: Option<(u32, u32)>,
    pub(crate) ext: Option<(&'static str, &'static str)>,
}

#[async_trait::async_trait]
pub(crate) trait MediaWorker: Send + Sync {
    async fn detect_ext(&self, source: &Arc<dyn TempData>) -> Option<(&'static str, &'static str)>;
    async fn split_pdf_to_png_pages(
        &self,
        path: &Path,
    ) -> Result<Vec<Arc<dyn TempData>>, ProcessingError>;
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

    async fn split_pdf_to_png_pages(
        &self,
        path: &Path,
    ) -> Result<Vec<Arc<dyn TempData>>, ProcessingError> {
        let path = path.to_path_buf();
        let (workdir, pages) = tokio::task::spawn_blocking(
            move || -> Result<(std::path::PathBuf, Vec<std::path::PathBuf>), std::io::Error> {
                let workdir =
                    std::env::temp_dir().join(format!("storage_pdf_{}", uuid::Uuid::new_v4()));
                std::fs::create_dir_all(&workdir)?;
                let prefix = workdir.join("page");

                let output = std::process::Command::new("pdftoppm")
                    .arg("-png")
                    .arg(&path)
                    .arg(&prefix)
                    .output()?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let _ = std::fs::remove_dir_all(&workdir);
                    return Err(std::io::Error::other(format!(
                        "pdftoppm failed: {}",
                        stderr.trim()
                    )));
                }

                let mut pages: Vec<_> = std::fs::read_dir(&workdir)?
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
                    .collect();
                pages.sort();

                if pages.is_empty() {
                    let _ = std::fs::remove_dir_all(&workdir);
                    return Err(std::io::Error::other(
                        "pdftoppm produced no pages for pdf input",
                    ));
                }

                Ok((workdir, pages))
            },
        )
        .await
        .map_err(ProcessingError::PdfWorkerJoin)?
        .map_err(ProcessingError::SplitPdf)?;

        let mut out: Vec<Arc<dyn TempData>> = Vec::with_capacity(pages.len());
        for page in pages {
            let mut page_file = File::open(&page).await.map_err(ProcessingError::SplitPdf)?;
            let mut tf = TempFile::new()
                .await
                .map_err(|e| ProcessingError::SplitPdf(std::io::Error::other(e.to_string())))?;
            tokio::io::copy(&mut page_file, &mut tf)
                .await
                .map_err(ProcessingError::SplitPdf)?;
            tf.flush().await.map_err(ProcessingError::SplitPdf)?;
            tf.sync_all().await.map_err(ProcessingError::SplitPdf)?;
            tf.seek(std::io::SeekFrom::Start(0))
                .await
                .map_err(ProcessingError::SplitPdf)?;
            let file_temp = FileTempData::from_tempfile(tf)
                .await
                .map_err(ProcessingError::SplitPdf)?;
            out.push(Arc::new(file_temp));
        }

        let _ = tokio::fs::remove_dir_all(workdir).await;
        Ok(out)
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
                    None,
                    converted
                        .open_stream()
                        .await
                        .map_err(ProcessingError::UploadConverted)?,
                )
                .await
                .map_err(ProcessingError::UploadConverted)?;
        } else {
            if is_image {
                if let Some(path) = source.as_local_path() {
                    dims = Some(
                        tokio::task::spawn_blocking(move || match image::image_dimensions(&path) {
                            Ok(v) => Ok(v),
                            Err(_) => {
                                let file = std::fs::File::open(&path)?;
                                let reader = std::io::BufReader::new(file);
                                image::ImageReader::new(reader)
                                    .with_guessed_format()?
                                    .into_dimensions()
                            }
                        })
                        .await
                        .map_err(ProcessingError::DimensionWorkerJoin)?
                        .map_err(ProcessingError::ReadImageDimensions)?,
                    );
                } else {
                    let bytes = source
                        .read_all()
                        .await
                        .map_err(ProcessingError::ReadTempFile)?;
                    dims = Some(
                        tokio::task::spawn_blocking(move || {
                            image::ImageReader::new(Cursor::new(bytes))
                                .with_guessed_format()?
                                .into_dimensions()
                        })
                        .await
                        .map_err(ProcessingError::DimensionWorkerJoin)?
                        .map_err(ProcessingError::ReadImageDimensions)?,
                    );
                }
            }
            writer
                .write(
                    &upload_handle,
                    None,
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
