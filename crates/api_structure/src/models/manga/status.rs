use serde::{Deserialize, Serialize};

use crate::error::{ApiErr, ApiErrorType};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Status {
    Dropped,
    Hiatus,
    Ongoing,
    Completed,
    Upcoming,
}

impl TryFrom<u64> for Status {
    type Error = ApiErr;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Dropped),
            1 => Ok(Self::Hiatus),
            2 => Ok(Self::Ongoing),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Upcoming),
            _ => Err(ApiErr {
                message: Some("Couldnt find manga status".to_string()),
                cause: None,
                err_type: ApiErrorType::InternalError,
            }),
        }
    }
}

impl From<Status> for u64 {
    fn from(value: Status) -> Self {
        match value {
            Status::Dropped => 0,
            Status::Hiatus => 1,
            Status::Ongoing => 2,
            Status::Completed => 3,
            Status::Upcoming => 4,
        }
    }
}
