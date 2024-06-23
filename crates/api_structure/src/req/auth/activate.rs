use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ActivateRequest {
    pub key: String,
}
