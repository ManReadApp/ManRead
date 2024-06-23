use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

impl From<usize> for Gender {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Female,
            1 => Self::Male,
            _ => Self::Unknown,
        }
    }
}
