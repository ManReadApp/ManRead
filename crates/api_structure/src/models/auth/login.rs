use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginWithUsernameAndPassword {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginWithEmailAndPassword {
    pub email: String,
    pub password: String,
}
