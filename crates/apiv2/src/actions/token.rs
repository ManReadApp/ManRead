use std::sync::Arc;

use api_structure::v1::{ActivationTokenKind, TokenKind};
use db::{
    auth::{AuthTokenDBService, AuthUser, RecordData},
    error::DbResult,
};

pub struct TokenAction {
    pub(crate) token: Arc<AuthTokenDBService>,
}

impl TokenAction {
    /// Creates a new auth token which is used to reset passwords or verify email addresses(and set roles).
    pub async fn create_token(
        &self,
        user_id: Option<String>,
        kind: ActivationTokenKind,
    ) -> DbResult<()> {
        self.token.create(user_id, kind).await
    }

    /// admin tool to list all tokens
    pub async fn list_tokens(&self, page: u32, limit: u32) -> DbResult<Vec<RecordData<AuthUser>>> {
        self.token.list(page, limit).await
    }

    /// admin tool to delete a token
    pub async fn delete_token(&self, id: &str) -> DbResult<()> {
        self.token.delete(id).await
    }
}
