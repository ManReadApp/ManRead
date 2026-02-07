use actix_web::web::{Data, Json, ReqData};
use actix_web_httpauth::middleware::HttpAuthentication;
use api_structure::{
    models::auth::jwt::Claim,
    req::auth::{
        activate::ActivateRequest,
        login::LoginRequest,
        register::RegisterRequest,
        reset_password::{RequestResetPasswordRequest, ResetPasswordRequest},
        TokenRefreshRequest,
    },
    resp::auth::JWTsResponse,
};
use apistos::{
    actix::CreatedJson,
    api_operation,
    web::{scope, Scope},
};
use storage::FileId;

use crate::{
    actions::{auth::AuthAction, crytpo::validator},
    error::ApiResult,
};

#[api_operation(tag = "auth", summary = "Registers a user", description = r###""###)]
async fn signup(
    Json(data): Json<RegisterRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
    service
        .register(
            &data.email,
            data.name,
            &data.password,
            data.gender,
            data.birthdate,
            data.icon_temp_name.map(|v| FileId::new(v)),
        )
        .await
        .map(CreatedJson)
}

#[api_operation(tag = "auth", summary = "Logs in the user", description = r###""###)]
pub async fn signin(
    Json(data): Json<LoginRequest>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
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
) -> ApiResult<CreatedJson<JWTsResponse>> {
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
) -> ApiResult<CreatedJson<JWTsResponse>> {
    service.refresh(&data.refresh_token).await.map(CreatedJson)
}

#[api_operation(tag = "auth", summary = "Verifies the user", description = r###""###)]
pub(crate) async fn verify(
    Json(data): Json<ActivateRequest>,
    claim: ReqData<Claim>,
    service: Data<AuthAction>,
) -> ApiResult<CreatedJson<JWTsResponse>> {
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
                    apistos::web::resource("/verify-account").route(apistos::web::put().to(verify)),
                ),
        )
}
