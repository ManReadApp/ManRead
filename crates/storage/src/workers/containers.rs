use std::sync::Arc;

use crate::{error::StorageResult, temp::TempData, MangaBundleMetadata, StorageError};

pub(crate) const CHAPTER_MAGIC: &[u8; 8] = b"MRCHAP01";
pub(crate) const MANGA_MAGIC: &[u8; 8] = b"MRMANG01";
const MAX_CONTAINER_ENTRIES: usize = 10_000;
const MAX_MANGA_METADATA_BYTES: u64 = 1024 * 1024;

pub(crate) enum ContainerPayload {
    SingleFile(Arc<dyn TempData>),
    Chapter(Vec<Arc<dyn TempData>>),
    Manga {
        metadata: Arc<dyn TempData>,
        chapter_image_indexes: Vec<Vec<u32>>,
        images: Vec<Arc<dyn TempData>>,
    },
}

#[async_trait::async_trait]
pub(crate) trait ContainerWorker: Send + Sync {
    async fn extract_payload(&self, source: Arc<dyn TempData>) -> StorageResult<ContainerPayload>;
}

pub(crate) struct MagicContainerWorker;

impl MagicContainerWorker {
    fn invalid_data(message: &'static str) -> StorageError {
        StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ))
    }

    async fn read_u32_at(source: &Arc<dyn TempData>, offset: u64) -> StorageResult<u32> {
        let bytes = source.read_at(offset, 4).await.map_err(StorageError::Io)?;
        let arr: [u8; 4] = bytes.as_slice().try_into().map_err(|_| {
            StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "missing u32",
            ))
        })?;
        Ok(u32::from_le_bytes(arr))
    }

    async fn extract_blob_sequence(
        source: &Arc<dyn TempData>,
        mut cursor: u64,
        count: usize,
    ) -> StorageResult<(Vec<Arc<dyn TempData>>, u64)> {
        let total = source.len().await.map_err(StorageError::Io)?;
        if count > MAX_CONTAINER_ENTRIES {
            return Err(Self::invalid_data("container contains too many files"));
        }
        let min_header_bytes = (count as u64).saturating_mul(4);
        if cursor > total || min_header_bytes > (total - cursor) {
            return Err(Self::invalid_data("container count exceeds payload bounds"));
        }

        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let len = Self::read_u32_at(source, cursor).await? as u64;
            cursor += 4;
            if cursor > total || len > (total - cursor) {
                return Err(StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "container blob out of range",
                )));
            }
            out.push(source.slice(cursor, len).map_err(StorageError::Io)?);
            cursor += len;
        }
        Ok((out, cursor))
    }

    async fn extract_chapter(source: &Arc<dyn TempData>) -> StorageResult<Vec<Arc<dyn TempData>>> {
        let count = Self::read_u32_at(source, CHAPTER_MAGIC.len() as u64).await? as usize;
        let (files, _) =
            Self::extract_blob_sequence(source, (CHAPTER_MAGIC.len() + 4) as u64, count).await?;
        Ok(files)
    }

    async fn extract_manga(
        source: &Arc<dyn TempData>,
    ) -> StorageResult<(Arc<dyn TempData>, Vec<Vec<u32>>, Vec<Arc<dyn TempData>>)> {
        let mut cursor = MANGA_MAGIC.len() as u64;
        let total = source.len().await.map_err(StorageError::Io)?;

        let metadata_len = Self::read_u32_at(source, cursor).await? as u64;
        cursor += 4;
        if metadata_len > MAX_MANGA_METADATA_BYTES {
            return Err(Self::invalid_data("container metadata exceeds limit"));
        }
        if cursor > total || metadata_len > (total - cursor) {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "container metadata out of range",
            )));
        }
        let metadata = source
            .slice(cursor, metadata_len)
            .map_err(StorageError::Io)?;
        cursor += metadata_len;

        let metadata_bytes = metadata.read_all().await.map_err(StorageError::Io)?;
        let metadata_struct: MangaBundleMetadata =
            bincode::deserialize(&metadata_bytes).map_err(|e| {
                StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid metadata: {e}"),
                ))
            })?;

        let image_count = Self::read_u32_at(source, cursor).await? as usize;
        cursor += 4;
        let (images, _) = Self::extract_blob_sequence(source, cursor, image_count).await?;

        for chapter in &metadata_struct.chapter_image_indexes {
            for idx in chapter {
                if (*idx as usize) >= images.len() {
                    return Err(StorageError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "metadata references out-of-range image index",
                    )));
                }
            }
        }

        Ok((metadata, metadata_struct.chapter_image_indexes, images))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        temp::{MemoryTempData, TempData},
        StorageError,
    };

    use super::{ContainerWorker, MagicContainerWorker, CHAPTER_MAGIC, MANGA_MAGIC};

    #[tokio::test]
    async fn chapter_container_rejects_excessive_count() {
        let mut payload = Vec::new();
        payload.extend_from_slice(CHAPTER_MAGIC);
        payload.extend_from_slice(&u32::MAX.to_le_bytes());
        let source: Arc<dyn TempData> = Arc::new(MemoryTempData::from_bytes(payload));

        let err = match MagicContainerWorker.extract_payload(source).await {
            Ok(_) => panic!("container should be rejected"),
            Err(err) => err,
        };

        match err {
            StorageError::Io(ioe) => assert_eq!(ioe.kind(), std::io::ErrorKind::InvalidData),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn manga_container_rejects_large_metadata() {
        let mut payload = Vec::new();
        payload.extend_from_slice(MANGA_MAGIC);
        payload.extend_from_slice(&(1024_u32 * 1024 + 1).to_le_bytes());
        let source: Arc<dyn TempData> = Arc::new(MemoryTempData::from_bytes(payload));

        let err = match MagicContainerWorker.extract_payload(source).await {
            Ok(_) => panic!("container should be rejected"),
            Err(err) => err,
        };

        match err {
            StorageError::Io(ioe) => assert_eq!(ioe.kind(), std::io::ErrorKind::InvalidData),
            other => panic!("unexpected error: {other}"),
        }
    }
}

#[async_trait::async_trait]
impl ContainerWorker for MagicContainerWorker {
    async fn extract_payload(&self, source: Arc<dyn TempData>) -> StorageResult<ContainerPayload> {
        let head = source.read_head(8).await.map_err(StorageError::Io)?;
        if head.len() < 8 {
            return Ok(ContainerPayload::SingleFile(source));
        }

        if head.as_slice() == CHAPTER_MAGIC {
            return Ok(ContainerPayload::Chapter(
                Self::extract_chapter(&source).await?,
            ));
        }

        if head.as_slice() == MANGA_MAGIC {
            let (metadata, chapter_image_indexes, images) = Self::extract_manga(&source).await?;
            return Ok(ContainerPayload::Manga {
                metadata,
                chapter_image_indexes,
                images,
            });
        }

        Ok(ContainerPayload::SingleFile(source))
    }
}
