use serde::{Deserialize, Serialize};

use crate::models::auth::login::{LoginWithEmailAndPassword, LoginWithUsernameAndPassword};

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum LoginRequest {
    Username(LoginWithUsernameAndPassword),
    Email(LoginWithEmailAndPassword),
}
