use std::sync::PoisonError;

use super::ApiError;

impl<Guard> From<PoisonError<Guard>> for ApiError {
    fn from(value: PoisonError<Guard>) -> Self {
        ApiError::PoisonError(value.to_string())
    }
}
