use crate::env::config::Config;
use crate::errors::{ApiError, ApiResult};
use actix_web::web;
use actix_web::web::Data;
use image::io::Reader as ImageReader;
use image::{guess_format, ImageFormat};
use std::fs::File;
use std::io::{Cursor, Write};

#[cfg(feature = "content-type-from-filename")]
pub fn get_content_type_from_filename(file_name: &str) -> Option<ImageFormat> {
    let ext = if file_name.contains('.') {
        file_name.split('.').last()
    } else {
        None
    };

    match ext {
        Some(v) => ImageFormat::from_extension(v),
        None => None,
    }
}

pub async fn write_file(
    filename: String,
    _old_file_name: &str,
    mut data: Vec<u8>,
    config: &Data<Config>,
) -> ApiResult<String> {
    #[cfg(feature = "content-type-from-filename")]
    let content_type = get_content_type_from_filename(_old_file_name);
    #[cfg(not(feature = "content-type-from-filename"))]
    let content_type = None;
    let mut content_type = match content_type {
        None => guess_format(&data)?,
        Some(v) => v,
    };

    // checks if image is broken
    let image = ImageReader::with_format(Cursor::new(data.clone()), content_type).decode()?;

    // converts the image types
    let allowed = [ImageFormat::Gif, ImageFormat::Jpeg, ImageFormat::Qoi];
    if !allowed.contains(&content_type) {
        let mut cursor = Cursor::new(Vec::new());
        let new_format = if content_type == ImageFormat::Png {
            ImageFormat::Qoi
        } else {
            ImageFormat::Jpeg
        };
        image
            .write_to(&mut cursor, new_format)
            .map_err(ApiError::write_error)?;
        data = cursor.into_inner();
        content_type = new_format;
    }

    let path = config.root_folder.join("temp");

    let file_name = format!("{}.{}", filename, get_extension(&content_type));
    let mut file = File::create(path.join(&file_name)).map_err(ApiError::write_error)?;
    web::block(move || file.write_all(&data))
        .await
        .map_err(ApiError::write_error)?
        .map_err(ApiError::write_error)?;
    Ok(file_name)
}

pub fn get_extension(content_type: &ImageFormat) -> String {
    let mut extension = content_type.extensions_str()[0];
    if extension == "jpg" {
        extension = "jpeg";
    }
    extension.to_string()
}
