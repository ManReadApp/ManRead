use actix_web::web::{Data, Json};
use api_structure::{
    models::auth::{jwt::Claim, role::Role},
    req::auth::register::RegisterRequest,
    resp::auth::JWTsResponse,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    models::user::UserDBService,
    services::{
        auth::CryptoService,
        file::{FileService, TempFile},
    },
    Config,
};

use std::{
    fs,
    path::{Path, PathBuf},
};

fn get_random_image(folder: &Path) -> Option<TempFile> {
    let extensions = ["png", "gif", "jpeg", "jpg"];

    let files: Vec<PathBuf> = fs::read_dir(folder)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;

            if !file_name.starts_with('.') {
                let ext = path.extension()?.to_str()?.to_lowercase();
                if extensions.contains(&ext.as_str()) {
                    return Some(path);
                }
            }
            None
        })
        .collect();
    use rand::prelude::IndexedRandom;
    files
        .choose(&mut rand::rng())
        .map(|path| TempFile::copy_file(path))
}

#[api_operation(tag = "auth", summary = "Registers a user", description = r###""###)]
pub(crate) async fn exec(
    Json(data): Json<RegisterRequest>,
    file_service: Data<FileService>,
    user_service: Data<UserDBService>,
    crypto_service: Data<CryptoService>,
    config: Data<Config>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    if user_service.email_exists(&data.email).await {
        return Err(ApiError::EmailExists);
    }
    if user_service.name_exists(&data.name).await {
        return Err(ApiError::NameExists);
    }
    let pw_hash = crypto_service.hash_password(&data.password)?;
    let file = match data.icon_temp_name {
        Some(v) => file_service.take(&v),
        None => get_random_image(&config.root_folder.join("cover_templates"))
            .ok_or(ApiError::NoCoverTemplatesFound),
    }?;

    // TODO: add file to array on fail
    let user = user_service
        .new_user(
            data.name,
            data.email.to_lowercase(),
            pw_hash,
            file.ext(),
            data.birthdate,
            data.gender as u32,
        )
        .await?;
    file.move_to(
        config.root_folder.as_path(),
        "users/icon",
        &user.id.id().to_string(),
    );
    Ok(CreatedJson(JWTsResponse {
        access_token: crypto_service.encode_claim(&Claim::new_access(
            user.id.id().to_string(),
            Role::NotVerified,
        ))?,
        refresh_token: crypto_service.encode_claim(&Claim::new_refresh(
            user.id.id().to_string(),
            Role::NotVerified,
        ))?,
    }))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/register").route(apistos::web::put().to(exec))
}
