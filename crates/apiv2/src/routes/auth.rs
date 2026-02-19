use actix_web::web::{Data, Json, ReqData};
use actix_web_grants::AuthorityGuard;
use actix_web_httpauth::middleware::HttpAuthentication;
use api_structure::{
    req::LoginRequest,
    v1::{
        ActivateRequest, Claim, JwTsResponse, JwtType, RegisterRequest,
        RequestResetPasswordRequest, ResetPasswordRequest, TokenRefreshRequest,
    },
    Permission,
};
use apistos::{
    actix::CreatedJson,
    api_operation,
    web::{scope, Scope},
};
use chrono::{DateTime, Utc};
use storage::FileId;

use crate::{
    actions::{auth::AuthAction, crytpo::validator},
    error::{ApiError, ApiResult},
};

#[api_operation(tag = "auth", summary = "Registers a user", description = r###""###)]
async fn signup(
    Json(data): Json<RegisterRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JwTsResponse>> {
    let birthdate = i64::try_from(data.birthdate)
        .ok()
        .and_then(DateTime::<Utc>::from_timestamp_millis)
        .ok_or(ApiError::invalid_input("invalid birthdate timestamp"))?;
    service
        .register(
            &data.email,
            data.name,
            &data.password,
            data.gender,
            birthdate,
            data.icon_temp_name.map(|v| FileId::new(v)),
        )
        .await
        .map(CreatedJson)
}

#[api_operation(tag = "auth", summary = "Logs in the user", description = r###""###)]
pub async fn signin(
    Json(data): Json<LoginRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JwTsResponse>> {
    service.login(data).await.map(CreatedJson)
}

#[api_operation(
    tag = "auth",
    summary = "Logs out every device of the user",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn logout(
    service: Data<AuthAction>,
    claim: ReqData<Claim>,
) -> ApiResult<Json<u8>> {
    if !matches!(claim.r#type, JwtType::AccessToken) {
        return Err(crate::error::ApiError::invalid_input(
            "Access token required",
        ));
    }
    service.logout(&claim).await?;
    Ok(Json(200))
}

#[api_operation(
    tag = "user",
    summary = "Updates the forgotten password",
    description = r###""###
)]
pub(crate) async fn verify_reset_password(
    Json(data): Json<ResetPasswordRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JwTsResponse>> {
    service.reset_password(data).await.map(CreatedJson)
}
#[api_operation(
    tag = "user",
    summary = "Requests the Reset of the password",
    description = r###""###
)]
pub(crate) async fn request_reset_password(
    Json(data): Json<RequestResetPasswordRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<u8>> {
    let uid = service.get_user_id(data.email, &data.ident).await?;
    service
        .request_reset_password(uid.id.id().to_string())
        .await?;
    Ok(CreatedJson(200))
}

#[api_operation(tag = "auth", summary = "Refreshes the token", description = r###""###)]
pub(crate) async fn refresh(
    Json(data): Json<TokenRefreshRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JwTsResponse>> {
    service.refresh(&data.refresh_token).await.map(CreatedJson)
}

#[api_operation(tag = "auth", summary = "Verifies the user", description = r###""###)]
pub(crate) async fn verify(
    Json(data): Json<ActivateRequest>,
    claim: ReqData<Claim>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JwTsResponse>> {
    service.verify(&data.key, &claim).await.map(CreatedJson)
}

pub fn register() -> Scope {
    apistos::web::scope("/auth")
        .service(apistos::web::resource("/register").route(apistos::web::put().to(signup)))
        .service(apistos::web::resource("/sign-in").route(apistos::web::post().to(signin)))
        .service(apistos::web::resource("/refresh").route(apistos::web::post().to(refresh)))
        .service(
            apistos::web::resource("/reset-password")
                .route(apistos::web::post().to(request_reset_password)),
        )
        .service(
            apistos::web::resource("/verify-reset-password")
                .route(apistos::web::put().to(verify_reset_password)),
        )
        .service(
            scope("")
                .wrap(HttpAuthentication::bearer(validator))
                .service(
                    apistos::web::resource("/sign-out").route(apistos::web::delete().to(logout)),
                )
                .service(
                    apistos::web::resource("/verify-account").route(
                        apistos::web::put()
                            .to(verify)
                            .guard(AuthorityGuard::new(Permission::Verify)),
                    ),
                ),
        )
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use actix_web::web::{Data, Json};
    use api_structure::v1::{Gender, RegisterRequest};
    use db::{init_db, DbConfig, MemoryDbConfig};
    use storage::{MemStorage, StorageSystem};

    use crate::actions::{auth::AuthAction, crytpo::CryptoService};

    use super::*;

    async fn test_auth_action() -> AuthAction {
        let db = init_db(DbConfig::Memory(MemoryDbConfig {
            namespace: format!("ns_route_auth_{}", helper::random_string(8)),
            database: format!("db_route_auth_{}", helper::random_string(8)),
        }))
        .await
        .expect("memory db should initialize");

        let root =
            std::env::temp_dir().join(format!("apiv2-route-auth-{}", helper::random_string(8)));
        tokio::fs::create_dir_all(&root)
            .await
            .expect("route auth temp root should be created");
        let storage = Arc::new(
            StorageSystem::new(PathBuf::as_path(&root), Arc::new(MemStorage::new()))
                .await
                .expect("memory storage should initialize"),
        );

        AuthAction {
            users: db.users,
            crypto: Arc::new(CryptoService::new(b"route-test-secret".to_vec())),
            token: db.tokens,
            fs: storage,
        }
    }

    #[actix_web::test]
    async fn signup_rejects_invalid_birthdate_timestamp() {
        let service = Data::new(test_auth_action().await);
        let response = signup(
            Json(RegisterRequest {
                email: "route@example.com".to_owned(),
                name: "route-user".to_owned(),
                password: "password".to_owned(),
                gender: Gender::Unknown,
                birthdate: u64::MAX,
                icon_temp_name: None,
            }),
            service,
        )
        .await;

        assert!(matches!(response, Err(ApiError::InvalidInput(_))));
    }
}
