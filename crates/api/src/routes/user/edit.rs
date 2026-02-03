use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use api_structure::{
    models::auth::{jwt::Claim, role::Permission},
    req::auth::register::UpdateUserRequest,
};
use apistos::{actix::CreatedJson, api_operation};

use crate::{
    error::{ApiError, ApiResult},
    models::user::UserDBService,
    services::{auth::CryptoService, file::FileService},
    Config,
};

#[api_operation(
    tag = "user",
    summary = "Edits a user",
    description = r###"Multiple edits are possible at once, but shouldnt be used. It will stop in the middle of the changes if there is a reason to stop. E.g. wrong password"###
)]
pub(crate) async fn exec(
    Json(data): Json<UpdateUserRequest>,
    claim: ReqData<Claim>,
    user_service: Data<UserDBService>,
    crypto_service: Data<CryptoService>,
    file_service: Data<FileService>,
    config: Data<Config>,
) -> ApiResult<CreatedJson<u8>> {
    if let Some(name) = data.name {
        user_service.replace_names(&claim.id, name).await?;
    }

    if let Some((old, new)) = data.password {
        let user = user_service.get_by_id(&claim.id).await?;
        let verify = crypto_service.verify_hash(old, user.data.password);
        if verify {
            user_service
                .set_password(&claim.id, crypto_service.hash_password(&new)?)
                .await?;
        } else {
            return Err(ApiError::PasswordIncorrect);
        }
    }

    if let Some(icon_temp_name) = data.icon_temp_name {
        let temp = file_service.take(&icon_temp_name)?;
        user_service.replace_icon_ext(&claim.id, temp.ext()).await?;
        temp.move_to(&config.root_folder, "users/icon", &claim.id);
    }

    if let Some(description) = data.description {
        user_service
            .replace_description(&claim.id, description)
            .await?;
    }

    if let Some(links) = data.links {
        user_service.replace_links(&claim.id, links).await?;
    }

    if let Some(thumbnail) = data.thumbnail {
        let temp = file_service.take(&thumbnail)?;
        user_service
            .replace_thumb_ext(&claim.id, temp.ext())
            .await?;
        temp.move_to(&config.root_folder, "users/banner", &claim.id);
    }
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/edit").route(
        apistos::web::put()
            .to(exec)
            .guard(AuthorityGuard::new(Permission::Read)),
    )
}
