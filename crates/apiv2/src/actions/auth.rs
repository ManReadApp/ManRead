use std::{sync::Arc, time::Duration};

use crate::{
    actions::crytpo::CryptoService,
    error::{ApiError, ApiResult},
};

use api_structure::{
    req::LoginRequest,
    v1::{ActivationTokenKind, Claim, Gender, JwTsResponse, JwtType, ResetPasswordRequest, Role},
    REFRESH_SECS,
};
use chrono::{DateTime, Utc};
use db::{
    auth::{is_token_valid, AuthTokenDBService, RecordData},
    error::DbResult,
    user::{UserDBService, UserRolePassword},
};
use storage::{FileBuilderExt as _, FileId, StorageSystem};

pub struct AuthAction {
    pub(crate) users: Arc<UserDBService>,
    pub(crate) crypto: Arc<CryptoService>,
    pub(crate) token: Arc<AuthTokenDBService>,
    pub(crate) fs: Arc<StorageSystem>,
}

impl AuthAction {
    fn validate_non_empty(field: &str, value: &str) -> ApiResult<()> {
        if value.trim().is_empty() {
            return Err(ApiError::invalid_input(&format!("{field} cannot be empty")));
        }
        Ok(())
    }

    fn role_from_db(value: u32) -> ApiResult<Role> {
        Role::try_from(value).map_err(|_| ApiError::write_error("invalid role value in database"))
    }

    /// creates a user if possible
    pub async fn register(
        &self,
        email: &String,
        name: String,
        password: &String,
        gender: Gender,
        birthdate: DateTime<Utc>,
        icon: Option<FileId>,
    ) -> ApiResult<JwTsResponse> {
        Self::validate_non_empty("email", email)?;
        Self::validate_non_empty("name", &name)?;
        Self::validate_non_empty("password", password)?;
        if self.users.email_exists(email).await {
            return Err(ApiError::EmailExists);
        }
        if self.users.name_exists(&name).await {
            return Err(ApiError::NameExists);
        }

        let pw_hash = self.crypto.hash_password(&password).await?;
        let cover_builder = self.fs.get_user_cover(icon).await?;

        let user = self
            .users
            .new_user(
                name,
                email.to_lowercase(),
                pw_hash,
                cover_builder.ext()?,
                birthdate,
                gender as u32,
            )
            .await?;
        cover_builder.build(&user.id.id().to_string()).await?;
        self.new_jwt(&user.id.id().to_string(), Role::NotVerified)
    }

    /// helper to generate jwt
    fn new_jwt(&self, user_id: &str, role: Role) -> ApiResult<JwTsResponse> {
        Ok(JwTsResponse {
            access_token: self
                .crypto
                .encode_claim(&Claim::new_access(user_id.to_owned(), role))?,
            refresh_token: self
                .crypto
                .encode_claim(&Claim::new_refresh(user_id.to_owned(), role))?,
        })
    }

    /// login action if credentials are valid
    pub async fn login(&self, data: LoginRequest) -> ApiResult<JwTsResponse> {
        let password = data.password();
        Self::validate_non_empty("password", &password)?;
        let user = match &data {
            LoginRequest::Username(l) => {
                Self::validate_non_empty("username", &l.username)?;
                self.users.get_by_name(&l.username).await
            }
            LoginRequest::Email(l) => {
                Self::validate_non_empty("email", &l.email)?;
                self.users.get_by_mail(&l.email).await
            }
        }?;
        let valid = self.crypto.verify_hash(password, user.data.password).await;
        if !valid {
            return Err(ApiError::PasswordIncorrect);
        }
        self.new_jwt(
            &user.id.id().to_string(),
            Self::role_from_db(user.data.role)?,
        )
    }

    pub async fn logout(&self, claim: &Claim) -> DbResult<()> {
        self.users.logout(&claim.id).await
    }

    /// generates refresh token if possible
    pub async fn refresh(&self, refresh_token: &str) -> ApiResult<JwTsResponse> {
        Self::validate_non_empty("refresh_token", refresh_token)?;
        let claim = self.crypto.get_claim(&refresh_token)?;
        if !matches!(claim.r#type, JwtType::RefreshToken) {
            return Err(ApiError::invalid_input("refresh token required"));
        }
        let (role, generated) = self.users.get_role_and_generated(claim.id.as_str()).await?;
        if generated > claim.exp as u128 - Duration::from_secs(REFRESH_SECS).as_millis() {
            return Err(ApiError::ExpiredToken);
        }
        self.new_jwt(&claim.id, role)
    }

    /// Creates a new token with Role::NotVerified
    pub async fn request_reset_password(&self, uid: String) -> DbResult<()> {
        self.token
            .create(
                Some(uid),
                ActivationTokenKind {
                    single: true,
                    kind: Role::NotVerified,
                },
            )
            .await
    }

    pub async fn get_user_id(
        &self,
        email: bool,
        ident: &String,
    ) -> ApiResult<RecordData<UserRolePassword>> {
        Ok(match email {
            true => self.users.get_by_mail(ident).await,
            false => self.users.get_by_name(ident).await,
        }?)
    }

    /// resets password with Role::NotVerified token
    pub async fn reset_password(&self, data: ResetPasswordRequest) -> ApiResult<JwTsResponse> {
        Self::validate_non_empty("ident", &data.ident)?;
        Self::validate_non_empty("key", &data.key)?;
        Self::validate_non_empty("password", &data.password)?;
        let user = self.get_user_id(data.email, &data.ident).await?;
        let token = self.token.find(&data.key).await?;
        is_token_valid(&token, &user.id.id().to_string())?;
        let role = match token.data.user.as_ref().map(|v| v.id()) == Some(user.id.id()) {
            true => Ok(token.data.get_kind().kind),
            false => Err(ApiError::WrongResetToken),
        }?;

        if role != Role::NotVerified {
            return Err(ApiError::WrongResetToken);
        };
        let hash = self.crypto.hash_password(&data.password).await?;
        self.users
            .set_password(&user.id.id().to_string(), hash)
            .await?;
        if token.data.get_kind().single {
            self.token.delete_(token).await?;
        }
        let role = Self::role_from_db(user.data.role)?;
        self.new_jwt(&user.id.id().to_string(), role)
    }

    /// uses a token and set user role
    pub async fn verify(&self, key: &str, claim: &Claim) -> ApiResult<JwTsResponse> {
        Self::validate_non_empty("key", key)?;
        let find = self.token.find(&key).await?;
        is_token_valid(&find, &claim.id.to_string())?;

        let kind = find.data.get_kind();

        self.users.set_role(claim.id.as_str(), kind.kind).await?;

        if kind.single {
            self.token.delete_(find).await?;
        }
        self.new_jwt(&claim.id, kind.kind)
    }
}
