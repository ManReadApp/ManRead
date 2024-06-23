use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ResetPasswordRequest {
    pub ident: String,
    pub email: bool,
    pub key: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct RequestResetPasswordRequest {
    pub ident: String,
    pub email: bool,
}
