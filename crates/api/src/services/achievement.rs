use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use actix_web::web::Data;
use apistos::ApiComponent;
use event_runner::{Event, ProcessType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{error::ApiResult, models::user::UserDBService};

#[derive(Serialize, Deserialize, ApiComponent, JsonSchema, Debug, Clone)]
pub enum Achievement {
    Joined,
    // 10, 100, 200, 500, 1000
    Read(u32),
    // 50, 100, 200, 500, 1000
    Favorited(u32),
    // 100, 200, 500, 1000, 10000
    Commented(u32),
    // 20, 100, 200, 500, 1000
    Reviewed(u32),
    // 10, 50, 100 200, 500
    Chatted(u32),
}

#[derive(Clone)]
pub struct AchievementService {
    user: Data<UserDBService>,
    is_running: Arc<Mutex<bool>>,
}
//TODO: repalce let _ with logging
impl AchievementService {
    pub fn new(user: Data<UserDBService>) -> Self {
        Self {
            user,
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    pub async fn joined_achievement(&self) {
        let item = self.user.all_joined_without_achievement().await;
        for item in item {
            let _ = self
                .set(&item.id.id().to_string(), Achievement::Joined)
                .await;
        }
        todo!()
    }
    pub async fn set(&self, id: &str, achievement: Achievement) -> ApiResult<()> {
        self.user.add_achievement(id, achievement).await
    }
}

impl Event for AchievementService {
    fn execute(
        &self,
    ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = ()> + Send>> {
        let s = self.clone();
        Box::pin(async move {
            s.joined_achievement().await;
        }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
    }

    fn cancel(&self) {}

    fn is_running(&self) -> bool {
        tokio::runtime::Handle::current().block_on(async { *self.is_running.lock().await })
    }

    fn parallel(&self) -> event_runner::ProcessType {
        ProcessType::Kind
    }

    fn set_handle(&self, _: tokio::task::JoinHandle<()>) {}

    fn wait_before_run(&self) -> Duration {
        Duration::from_secs(60 * 15)
    }

    fn rerun(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 15))
    }
}
