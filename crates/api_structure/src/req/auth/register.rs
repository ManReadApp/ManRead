use crate::models::auth::gender::Gender;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub birthdate: NaiveDate,
    pub gender: Gender,
    pub icon_temp_name: String,
}
