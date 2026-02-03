use std::{fs::File, io::Write as _};

use actix_multipart::{Field, Multipart};
use actix_web::web::{self, Data};
use actix_web_grants::AuthorityGuard;
use api_structure::models::auth::role::Permission;
use apistos::{actix::CreatedJson, api_operation};
use futures_util::{StreamExt, TryStreamExt as _};
use image::{guess_format, ImageFormat};

use crate::{
    error::{ApiError, ApiResult},
    services::file::FileService,
};

#[api_operation(
    tag = "image",
    skip_args = "payload",
    summary = "Upload an image or mulitple images",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn exec(
    mut payload: Multipart,
    file_service: Data<FileService>,
) -> ApiResult<CreatedJson<Vec<Vec<String>>>> {
    let mut register = vec![];
    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = match get_field_name(&field)? {
            Some(v) => v,
            None => continue,
        };
        match field_name.as_str() {
            "image[]" | "images[]" | "image" | "images" => {}
            _ => {
                //TODO: add txt, archive, pdf, psd
                return Err(ApiError::invalid_input(
                    "Invalid field name(only allows \"images\")",
                ));
            }
        }
        let old_file_name = field
            .content_disposition()
            .ok_or(ApiError::NoContentDisposition)?
            .get_filename()
            .unwrap_or_default()
            .to_owned();
        let (file, mut temp_file) = file_service.new_temp_file().await?;
        let header = process_data(&mut field, file).await?;
        temp_file.finish_writing(get_content_type(&old_file_name, &header)?)?;
        register.push((old_file_name, temp_file.id()));
        file_service.register_temp_file(temp_file);
    }

    Ok(CreatedJson(
        register.into_iter().map(|v| vec![v.0, v.1]).collect(),
    ))
}

async fn process_data(field: &mut Field, mut file: File) -> ApiResult<Vec<u8>> {
    let mut file_data = Vec::new();
    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(ApiError::multipart_read_error)?
    {
        extend_buffer(&mut file_data, &chunk);
        file = web::block(move || file.write_all(&chunk).map(|_| file))
            .await
            .map_err(ApiError::write_error)?
            .map_err(ApiError::write_error)?;
    }
    Ok(file_data)
}

fn get_field_name(field: &Field) -> ApiResult<Option<String>> {
    Ok(
        match field
            .content_disposition()
            .ok_or(ApiError::NoContentDisposition)?
            .get_name()
        {
            Some(v) => Some(v.to_string()),
            None => None,
        },
    )
}

/// Only the first bytes of the file are put into the buffer
/// For reading magic header
fn extend_buffer(buffer: &mut Vec<u8>, data: &[u8]) {
    const MAX_SIZE: usize = 32;

    let available_space = MAX_SIZE.saturating_sub(buffer.len());
    if available_space > 0 {
        let to_add = data.len().min(available_space);
        buffer.extend_from_slice(&data[..to_add]);
    }
}

fn get_content_type(_old_file_name: &str, buffer: &[u8]) -> ApiResult<ImageFormat> {
    #[cfg(feature = "content-type-from-filename")]
    let content_type = get_content_type_from_filename(_old_file_name);
    #[cfg(not(feature = "content-type-from-filename"))]
    let content_type = None;
    let content_type = match content_type {
        None => guess_format(buffer)?,
        Some(v) => v,
    };
    Ok(content_type)
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/upload").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::None)),
    )
}
