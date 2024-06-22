use serde::{Deserialize, Serialize};

use crate::error::{ApiErr, ApiErrorType};

#[derive(Serialize, Deserialize)]
pub enum Visibility {
    /// Everyone
    Visible,
    /// Admins,Coadmins, Mods, and Author
    Hidden,
    /// Admins,Coadmins, Mods
    AdminReview,
}

impl TryFrom<u64> for Visibility {
    type Error = ApiErr;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Visible),
            1 => Ok(Self::Hidden),
            2 => Ok(Self::AdminReview),
            _ => Err(ApiErr {
                message: Some("unknown visibility".to_string()),
                cause: None,
                err_type: ApiErrorType::InternalError,
            }),
        }
    }
}
