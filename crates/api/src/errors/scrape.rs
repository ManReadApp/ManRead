use crate::errors::ApiError;
use manread_scraper::ScrapeError;

impl From<ScrapeError> for ApiError {
    fn from(error: ScrapeError) -> Self {
        ApiError::Inner(error.0)
    }
}
