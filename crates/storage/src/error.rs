use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("processing semaphore closed")]
    SemaphoreClosed,
    #[error("seek temp file failed: {0}")]
    SeekTempFile(#[source] std::io::Error),
    #[error("read temp file failed: {0}")]
    ReadTempFile(#[source] std::io::Error),
    #[error("image worker join failed: {0}")]
    ImageWorkerJoin(#[source] tokio::task::JoinError),
    #[error("image conversion failed: {0}")]
    ImageConversion(#[source] image::ImageError),
    #[error("dimension worker join failed: {0}")]
    DimensionWorkerJoin(#[source] tokio::task::JoinError),
    #[error("pdf worker join failed: {0}")]
    PdfWorkerJoin(#[source] tokio::task::JoinError),
    #[error("read image dimensions failed: {0}")]
    ReadImageDimensions(#[source] image::ImageError),
    #[error("split pdf failed: {0}")]
    SplitPdf(#[source] std::io::Error),
    #[error("open temp file for upload failed: {0}")]
    OpenTempFileForUpload(#[source] std::io::Error),
    #[error("upload converted file failed: {0}")]
    UploadConverted(#[source] std::io::Error),
    #[error("upload temp file failed: {0}")]
    UploadTemp(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("handle not found")]
    HandleNotFound,
    #[error("no default image available")]
    NoDefaultImageAvailable,
    #[error("missing file extension")]
    MissingExtension,
    #[error("processing failed: {0}")]
    Processing(#[from] ProcessingError),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("temp file error: {0}")]
    TempFile(#[from] async_tempfile::Error),
}
pub type StorageResult<T> = Result<T, StorageError>;
