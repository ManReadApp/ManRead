use crate::env::config::{random_string, Config};
use crate::errors::{ApiError, ApiResult};
use crate::routes::image::save::write_file;
use actix_multipart::Multipart;
use actix_web::web::Data;
use api_structure::now_timestamp;
use futures_util::{StreamExt, TryStreamExt};

pub async fn upload_images(
    mut payload: Multipart,
    config: Data<Config>,
) -> ApiResult<Vec<(String, String)>> {
    let mut images: Vec<(String, String)> = vec![];
    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = match field
            .content_disposition()
            .ok_or(ApiError::NoContentDisposition)?
            .get_name()
        {
            Some(v) => v.to_string(),
            None => continue,
        };
        match field_name.as_str() {
            "image[]" => {
                let (file_name, temp_name) = match field
                    .content_disposition()
                    .ok_or(ApiError::NoContentDisposition)?
                    .get_filename()
                    .map(String::from)
                {
                    Some(v) => (
                        v.to_string(),
                        format!(
                            "{}-{}",
                            now_timestamp().expect("time went backwards").as_millis(),
                            random_string(32)
                        ),
                    ),
                    None => continue,
                };
                let mut file_data = vec![];
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(ApiError::multipart_read_error)?
                {
                    file_data.extend_from_slice(&chunk);
                }

                let name = write_file(temp_name, &file_name, file_data, &config).await?;
                images.push((file_name, name));
            }
            _ => {
                return Err(ApiError::invalid_input(
                    "Invalid field name(only allows \"images\")",
                ))
            }
        }
    }
    match images.is_empty() {
        true => Err(ApiError::invalid_input("No valid images given")),
        false => Ok(images),
    }
}
