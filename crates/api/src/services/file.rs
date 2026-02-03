use std::{
    collections::HashMap,
    fs::{read_dir, File},
    io::{Read as _, Write},
    num::ParseIntError,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::web;
use image::ImageFormat;

use crate::{
    error::{ApiError, ApiResult},
    init::random_string,
    Config,
};

pub struct FileService {
    files: Mutex<HashMap<String, TempFile>>,
    config: Arc<Config>,
}

#[derive(Clone)]
pub struct TempFile {
    path: PathBuf,
    id: String,
    time: u128,
    dims: Option<(u32, u32)>,
    extension: String,
    copy: bool,
}

impl TempFile {
    pub fn copy_file(path: &Path) -> Self {
        let mut ext = path
            .extension()
            .and_then(|v| v.to_str().map(|v| v.to_owned()))
            .unwrap_or_default();
        if ext == "jpg" {
            ext = "jpeg".to_owned();
        }
        let mut file = TempFile::new(path.to_path_buf(), &ext);
        file.copy = true;
        file
    }

    pub fn move_to(self, root: &Path, path: &str, name: &str) {
        let new_path = root.join(path).join(format!("{}.{}", name, self.extension));
        if self.copy {
            let _ = std::fs::copy(&self.path, &new_path);
        } else {
            let _ = std::fs::rename(&self.path, &new_path);
        }
        if self.extension == "qoi" {
            let new_path_avif = root.join(path).join(format!("{}.{}", name, "avif"));
            if let Ok(img) = image::open(new_path) {
                let _ = img.save_with_format(new_path_avif, ImageFormat::Avif);
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.dims.map(|v| v.0).unwrap_or_default()
    }
    pub fn height(&self) -> u32 {
        self.dims.map(|v| v.1).unwrap_or_default()
    }
    pub fn ext(&self) -> &str {
        &self.extension
    }
}

fn identify_apng(filename: &Path) -> std::io::Result<bool> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    let mut chunk = [0; 1024];

    while let Ok(bytes_read) = file.read(&mut chunk) {
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
        if let Some(idat_pos) = find_subslice(&buffer, b"IDAT") {
            if let Some(_actl_pos) = find_subslice(&buffer[..idat_pos], b"acTL") {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
    }

    Ok(false)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
impl TempFile {
    fn update_path(&mut self) -> ApiResult<()> {
        let new_name = self.generate_temp_file_name();
        let parent_dir = self.path.parent();

        let new_path = match parent_dir {
            Some(parent_dir) => parent_dir.join(new_name),
            None => unreachable!("Forgot to set parent dir"),
        };

        std::fs::rename(&self.path, &new_path).map_err(ApiError::write_error)?;

        self.path = new_path;
        Ok(())
    }

    fn process(&mut self) -> ApiResult<()> {
        //TODO: animated items should not be hashed & not converted. webp, apng, avif, gif. gif, avif, webp arent converted anyway and apng has a check
        match image::open(&self.path) {
            Ok(img) => {
                let ext = ImageFormat::from_extension(&self.extension).unwrap();
                let convert_to = match ext {
                    ImageFormat::Png
                    | ImageFormat::Tiff
                    | ImageFormat::Bmp
                    | ImageFormat::Qoi
                    | ImageFormat::Ico
                    | ImageFormat::Pcx
                    | ImageFormat::Tga
                    | ImageFormat::Farbfeld => ImageFormat::Qoi,
                    ImageFormat::Jpeg | ImageFormat::Dds => ImageFormat::Jpeg,
                    ImageFormat::Gif => ImageFormat::Gif,
                    ImageFormat::WebP => ImageFormat::WebP,
                    ImageFormat::Hdr | ImageFormat::Avif | ImageFormat::OpenExr => {
                        ImageFormat::Avif
                    }
                    _ => todo!("Add support for more formats"),
                };
                if ext != convert_to
                    && !(ext == ImageFormat::Png && identify_apng(&self.path).unwrap_or_default())
                {
                    let _ = img.save_with_format(&self.path, convert_to);
                    self.extension = convert_to.extensions_str()[0].to_owned();
                    if self.extension == "jpg" {
                        self.extension = "jpeg".to_owned();
                    }
                }
                self.dims = Some((img.width(), img.height()));
            }
            Err(_) => {}
        };
        self.update_path()
    }

    pub fn finish_writing(&mut self, content_type: ImageFormat) -> ApiResult<()> {
        if self.extension == "tmp" {
            self.extension = content_type.extensions_str()[0].to_owned();
            if self.extension == "jpg" {
                self.extension = "jpeg".to_owned();
            }
            self.update_path()?;
        }
        Ok(())
    }
    pub fn id(&self) -> String {
        format!("{}-{}", self.id, self.time)
    }
    /// The path does not exist yet and is just the name
    fn new(path: PathBuf, extension: &str) -> Self {
        let id = random_string(10);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            path: path.join(Self::generate_temp_file_name_init(&id, time, extension)),
            time,
            id,
            dims: None,
            extension: extension.to_owned(),
            copy: false,
        }
    }

    pub fn from_bytes(root: &Path, bytes: Vec<u8>, extension: &str) -> Self {
        let id = random_string(10);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = root.join(Self::generate_temp_file_name_init(&id, time, extension));
        File::create(&path).unwrap().write_all(&bytes).unwrap();
        let mut se = Self {
            path: path,
            time,
            id,
            dims: None,
            extension: extension.to_owned(),
            copy: false,
        };
        se.process().unwrap();
        se
    }

    fn generate_temp_file_name(&self) -> String {
        let dims = match self.dims {
            Some((width, height)) => format!("{}x{}", width, height),
            None => String::new(),
        };
        format!("{}-{}-{}.{}", self.id, self.time, dims, self.extension)
    }

    fn generate_temp_file_name_init(id: &str, time: u128, extension: &str) -> String {
        format!("{}-{}-.{}", id, time, extension)
    }

    /// Creates file at path
    async fn get_file(&self) -> ApiResult<File> {
        let path = self.path.clone();

        if path.exists() {
            return Err(ApiError::CannotSaveTempFile);
        }
        let path = self.path.clone();

        Ok(web::block(move || File::create(path)).await?.unwrap())
    }

    /// reads temp file
    fn read(path: &Path) -> ApiResult<Self> {
        if !path.is_file() {
            return Err(ApiError::TempFileNotFound);
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let (name, ext) = name
            .rsplit_once(".")
            .ok_or(ApiError::NoFileExtensionForTempFile)?;
        let mut seg = name.split("-");
        let id = seg.next().ok_or(ApiError::MalformedTempFilename)?;
        let time = seg
            .next()
            .ok_or(ApiError::MalformedTempFilename)?
            .parse::<u128>()
            .map_err(|_| ApiError::MalformedTempFilename)?;
        let dim = seg.next();
        let dims = dim
            .map(|v| v.split_once("x").ok_or(ApiError::MalformedTempFilename))
            .transpose()?
            .map(|v| Ok((v.0.parse::<u32>()?, v.1.parse::<u32>()?)))
            .transpose()
            .map_err(|_: ParseIntError| ApiError::MalformedTempFilename)?;
        Ok(Self {
            time,
            path: path.to_path_buf(),
            id: id.to_owned(),
            dims,
            extension: ext.to_owned(),
            copy: false,
        })
    }
}

impl FileService {
    pub fn new(config: Arc<Config>) -> Self {
        let path = config.root_folder.join("temp");
        let paths = read_dir(path)
            .map(|v| {
                v.filter_map(|v| v.ok())
                    .map(|v| TempFile::read(Path::new(v.file_name().to_str().unwrap_or_default())))
                    .filter_map(|v| v.ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            config,
            files: Mutex::new(paths.into_iter().map(|v| (v.id(), v)).collect()),
        }
    }
    /// Creates new file
    pub async fn new_temp_file(&self) -> ApiResult<(File, TempFile)> {
        let temp_file = TempFile::new(self.config.root_folder.join("temp"), "tmp");
        Ok((temp_file.get_file().await?, temp_file))
    }

    pub fn register_temp_file(&self, mut temp_file: TempFile) {
        let _ = temp_file.process();
        self.files.lock().unwrap().insert(temp_file.id(), temp_file);
    }

    pub fn take(&self, id: &str) -> ApiResult<TempFile> {
        let file = self
            .files
            .lock()
            .unwrap()
            .remove(id)
            .ok_or(ApiError::TempFileNotFound)?;
        Ok(file)
    }
}
