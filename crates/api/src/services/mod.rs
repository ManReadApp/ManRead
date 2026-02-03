use std::time::Duration;

pub mod achievement;
pub mod auth;
pub mod file;
pub mod scraper;

#[async_trait::async_trait]
pub trait BackgroundService {
    async fn run(&self);
    fn when(&self) -> Duration;
}
