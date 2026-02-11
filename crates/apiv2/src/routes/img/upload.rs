use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web_grants::AuthorityGuard;
use api_structure::Permission;
use apistos::{actix::CreatedJson, api_operation};
use futures_util::{StreamExt, TryStreamExt as _};
use storage::{RegisterTempResult, StorageSystem, TempFile};
use tokio::io::AsyncWriteExt;

use crate::error::{ApiError, ApiResult};

#[api_operation(
    tag = "image",
    skip_args = "payload",
    summary = "Upload an image or mulitple images",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn exec(
    mut payload: Multipart,
    file_service: Data<StorageSystem>,
) -> ApiResult<CreatedJson<Vec<Vec<String>>>> {
    let mut register = vec![];
    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = match get_field_name(&field)? {
            Some(v) => v,
            None => continue,
        };
        match field_name.as_str() {
            "file[]" | "files[]" | "file" | "files" => {}
            _ => {
                return Err(ApiError::InvalidInput(
                    "Invalid field name(only allows \"files\")".to_owned(),
                ));
            }
        }
        let old_file_name = field
            .content_disposition()
            .ok_or(ApiError::NoContentDisposition)?
            .get_filename()
            .unwrap_or_default()
            .to_owned();
        let mut temp_file = file_service.new_temp_file().await?;
        process_data(&mut field, &mut temp_file).await?;
        match file_service.register_temp_file(temp_file).await? {
            RegisterTempResult::File(id) => register.push((old_file_name, id)),
            RegisterTempResult::Chapter(ids) => {
                for (idx, id) in ids.into_iter().enumerate() {
                    register.push((format!("{old_file_name}#p{}", idx), id));
                }
            }
            RegisterTempResult::Manga(manga) => {
                register.push((format!("{old_file_name}#meta"), manga.metadata));
                for (idx, id) in manga.images.into_iter().enumerate() {
                    register.push((format!("{old_file_name}#i{}", idx), id));
                }
            }
        }
    }

    Ok(CreatedJson(
        register
            .into_iter()
            .map(|v| vec![v.0, v.1.inner()])
            .collect(),
    ))
}

async fn process_data(field: &mut Field, file: &mut TempFile) -> ApiResult<()> {
    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(ApiError::multipart_read_error)?
    {
        file.write_all(&chunk)
            .await
            .map_err(|e| ApiError::write_error(e))?;
    }
    Ok(())
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

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/upload").route(
        apistos::web::post()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::None)),
    )
}
