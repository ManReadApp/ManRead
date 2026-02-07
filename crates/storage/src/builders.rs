use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{backends::StorageWriter, error::StorageResult, FileBuilderExt, StorageError};

pub struct FileBuilder {
    pub(crate) temp_id: String,
    pub(crate) target_id: PathBuf,
    pub(crate) ext: Option<(&'static str, &'static str)>,
    pub(crate) dims: Option<(u32, u32)>,
    pub(crate) allowed_drop: bool,
    pub(crate) writer: Arc<dyn StorageWriter>,
}

impl FileBuilder {
    pub fn manga_page(
        self,
        manga_id: &str,
        chapter_id: &str,
        version_id: &str,
    ) -> MangaPageFileBuilder {
        MangaPageFileBuilder {
            b: self.add_path(format!("mangas/{manga_id}/{chapter_id}/{version_id}")),
        }
    }

    fn width_impl(&self) -> Option<u32> {
        self.dims.map(|(w, _)| w)
    }

    fn height_impl(&self) -> Option<u32> {
        self.dims.map(|(_, h)| h)
    }

    fn ext_impl(&self) -> StorageResult<&str> {
        self.ext.map(|v| v.1).ok_or(StorageError::MissingExtension)
    }

    async fn build(mut self) -> StorageResult<()> {
        self.allowed_drop = true;
        if let Some((_, ext)) = self.ext {
            let ext = ext.strip_prefix('.').unwrap_or(ext);
            if !ext.is_empty() {
                self.target_id.set_extension(ext);
            }
        }
        self.writer
            .rename(&self.temp_id, &self.target_id.to_string_lossy().to_owned())
            .await;
        Ok(())
    }

    fn add_path<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.target_id.push(path);
        self
    }
}

impl Drop for FileBuilder {
    fn drop(&mut self) {
        if !self.allowed_drop {
            log::error!("FileBuilder dropped without being built",);
        }
    }
}

macro_rules! builder_wrapper {
    ($name:ident, $func:ident) => {
        pub struct $name {
            b: FileBuilder,
        }

        impl FileBuilderExt for $name {
            fn width(&self) -> Option<u32> {
                self.b.width_impl()
            }

            fn height(&self) -> Option<u32> {
                self.b.height_impl()
            }

            fn ext(&self) -> StorageResult<&str> {
                self.b.ext_impl()
            }
        }
        impl From<FileBuilder> for $name {
            fn from(b: FileBuilder) -> Self {
                Self { b: $func(b) }
            }
        }
    };
}

builder_wrapper!(MangaPageFileBuilder, from_no_change);
builder_wrapper!(UserCoverFileBuilder, from_user_cover);
builder_wrapper!(CoverFileBuilder, from_cover);

fn from_no_change(fb: FileBuilder) -> FileBuilder {
    fb
}

fn from_user_cover(fb: FileBuilder) -> FileBuilder {
    fb.add_path("users/icon")
}

fn from_cover(fb: FileBuilder) -> FileBuilder {
    fb.add_path("covers")
}

impl UserCoverFileBuilder {
    pub async fn build(self, id: &str) -> StorageResult<()> {
        self.b.add_path(id).build().await
    }
}

impl CoverFileBuilder {
    pub async fn build(self, id: &str) -> StorageResult<()> {
        self.b.add_path(id).build().await
    }
}

impl MangaPageFileBuilder {
    pub async fn build(self, id: usize) -> StorageResult<()> {
        self.b.add_path(id.to_string()).build().await
    }
}
