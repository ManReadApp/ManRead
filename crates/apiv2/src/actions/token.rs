use std::sync::Arc;

use api_structure::v1::ActivationTokenKind;
use db::auth::{AuthTokenDBService, AuthUser, RecordData};

use crate::error::{ApiError, ApiResult};

pub struct TokenAction {
    pub(crate) token: Arc<AuthTokenDBService>,
}

impl TokenAction {
    /// Creates a new auth token which is used to reset passwords or verify email addresses(and set roles).
    pub async fn create_token(
        &self,
        user_id: Option<String>,
        kind: ActivationTokenKind,
    ) -> ApiResult<()> {
        if let Some(user_id) = &user_id {
            if user_id.trim().is_empty() {
                return Err(ApiError::invalid_input("user_id cannot be empty"));
            }
        }
        self.token.create(user_id, kind).await?;
        Ok(())
    }

    /// admin tool to list all tokens
    pub async fn list_tokens(&self, page: u32, limit: u32) -> ApiResult<Vec<RecordData<AuthUser>>> {
        if page == 0 {
            return Err(ApiError::invalid_input("page must be >= 1"));
        }
        if limit == 0 {
            return Err(ApiError::invalid_input("limit must be >= 1"));
        }
        Ok(self.token.list(page, limit).await?)
    }

    /// admin tool to delete a token
    pub async fn delete_token(&self, id: &str) -> ApiResult<()> {
        if id.trim().is_empty() {
            return Err(ApiError::invalid_input("id cannot be empty"));
        }
        self.token.delete(id).await?;
        Ok(())
    }
}
