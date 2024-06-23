use crate::env::config::Config;
use crate::errors::ApiResult;
use crate::services::crypto_service::CryptoService;
use crate::services::db::user::UserDBService;
use actix_web::post;
use actix_web::web::{Data, Json};
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::models::auth::jwt::Claim;
use api_structure::models::auth::role::Role;
use api_structure::req::auth::register::RegisterRequest;
use api_structure::resp::auth::JWTsResponse;

#[post("/sign_up")]
async fn sign_up_route(
    Json(data): Json<RegisterRequest>,
    crypto: Data<CryptoService>,
    config: Data<Config>,
    db: Data<UserDBService>,
) -> ApiResult<Json<JWTsResponse>> {
    if !config
        .root_folder
        .join("temp")
        .join(&data.icon_temp_name)
        .exists()
    {
        return Err(ApiErr {
            message: Some("File does not exist".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into());
    }
    if db.email_exists(&data.email.to_lowercase()).await {
        return Err(ApiErr {
            message: Some("Email already exists".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into());
    }

    if db.username_exists(&data.name).await {
        return Err(ApiErr {
            message: Some("Username already exists".to_string()),
            cause: None,
            err_type: ApiErrorType::InvalidInput,
        }
        .into());
    }

    let ext = data.icon_temp_name.split('.').collect::<Vec<_>>();
    let ext = match ext.get(1) {
        Some(v) => Ok(v),
        None => Err(ApiErr {
            cause: None,
            message: Some("Invalid file name".to_string()),
            err_type: ApiErrorType::InvalidInput,
        }),
    }?;
    let user = db
        .new_user(
            data.name,
            data.email.to_lowercase(),
            crypto.hash_password(&data.password)?,
            ext.to_string(),
            data.birthdate,
            data.gender,
        )
        .await?;

    let id = user.id.id().to_string();

    let name = format!("{}.{}", id, ext);

    std::fs::rename(
        config.root_folder.join("temp").join(data.icon_temp_name),
        config.root_folder.join("users").join("icon").join(name),
    )?;
    Ok(Json(JWTsResponse {
        access_token: crypto.encode_claim(&Claim::new_access(id.clone(), Role::NotVerified)?)?,
        refresh_token: crypto.encode_claim(&Claim::new_refresh(id.clone(), Role::NotVerified)?)?,
    }))
}
