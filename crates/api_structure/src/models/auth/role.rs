use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Role {
    NotVerified = 0,
    User = 1,
    Author = 2,
    Moderator = 3,
    CoAdmin = 4,
    Admin = 5,
}

impl From<u32> for Role {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::User,
            2 => Self::Author,
            3 => Self::Moderator,
            4 => Self::CoAdmin,
            5 => Self::Admin,
            _ => Self::NotVerified,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::NotVerified => write!(f, "Undifined"),
            Role::User => write!(f, "User"),
            Role::Moderator => write!(f, "Moderator"),
            Role::CoAdmin => write!(f, "Co-Admin"),
            Role::Admin => write!(f, "Admin"),
            Role::Author => write!(f, "Author"),
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(role: &str) -> Result<Self, Self::Err> {
        Ok(match role {
            "Author" => Self::Author,
            "Admin" => Self::Admin,
            "Co-Admin" => Self::CoAdmin,
            "Moderator" => Self::Moderator,
            "User" => Self::User,
            _ => Self::NotVerified,
        })
    }
}
