use std::{
    path::{Component, Path, PathBuf},
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
        validate_target_key(&self.target_id)?;
        let key = self.target_id.to_string_lossy().to_string();
        self.writer.rename(&self.temp_id, &key).await?;
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

fn validate_target_key(path: &Path) -> StorageResult<()> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path cannot be empty",
        )));
    }

    for comp in path.components() {
        match comp {
            Component::Normal(_) => {}
            _ => {
                return Err(StorageError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "target path must be a safe relative path",
                )));
            }
        }
    }
    Ok(())
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

builder_wrapper!(MangaPageFileBuilder, from_manga);
builder_wrapper!(UserCoverFileBuilder, from_user_cover);
builder_wrapper!(CoverFileBuilder, from_cover);
builder_wrapper!(ArtFileBuilder, from_art);
builder_wrapper!(UserBannerBuilder, from_banner);

fn from_manga(fb: FileBuilder) -> FileBuilder {
    unreachable!()
}

fn from_user_cover(fb: FileBuilder) -> FileBuilder {
    fb.add_path("users/icon")
}
fn from_banner(fb: FileBuilder) -> FileBuilder {
    fb.add_path("users/banner")
}

fn from_cover(fb: FileBuilder) -> FileBuilder {
    fb.add_path("covers")
}

fn from_art(fb: FileBuilder) -> FileBuilder {
    fb.add_path("arts")
}

impl UserCoverFileBuilder {
    pub async fn build(self, id: &str) -> StorageResult<()> {
        self.b.add_path(id).build().await
    }
}

impl UserBannerBuilder {
    pub async fn build(self, id: &str) -> StorageResult<()> {
        self.b.add_path(id).build().await
    }
}

impl ArtFileBuilder {
    pub async fn build(self, id: &str, index: usize) -> StorageResult<()> {
        self.b.add_path(format!("{}_{}", id, index)).build().await
    }
}

impl CoverFileBuilder {
    pub async fn build(self, id: &str, index: usize) -> StorageResult<()> {
        let id = if index == 0 {
            id.to_owned()
        } else {
            format!("{}_{}", id, index)
        };
        self.b.add_path(id).build().await
    }
}

impl MangaPageFileBuilder {
    pub async fn build(self, page: usize) -> StorageResult<()> {
        self.b.add_path(page.to_string()).build().await
    }
}
