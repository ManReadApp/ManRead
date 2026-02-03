use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use actix_web::web::Data;
use manga_scraper::init::Services;
use tokio::time::sleep;

use crate::{
    models::user::UserDBService,
    services::{
        scraper::{ScraperService, ScraperService2},
        BackgroundService,
    },
    Config,
};

pub struct BackgroundServiceRunner {
    service: Vec<Box<dyn BackgroundService + 'static + Send + Sync>>,
    runs: Arc<Mutex<HashMap<usize, Instant>>>,
}

fn subtract_durations(a: Duration, b: Duration) -> Duration {
    if a > b {
        a - b
    } else {
        Duration::new(0, 0)
    }
}

impl BackgroundServiceRunner {
    pub fn new(user: Data<UserDBService>, config: Arc<Config>, services: Arc<Services>) -> Self {
        let mut service: Vec<Box<dyn BackgroundService + Send + Sync>> = vec![];
        //service.push(Box::new(AchievementService::new(user)));
        service.push(Box::new(ScraperService::new(services.clone())));
        service.push(Box::new(ScraperService2::new(config, services)));

        Self {
            service,
            runs: Default::default(),
        }
    }

    pub async fn run(&self) {
        loop {
            let next = {
                let runs = self.runs.lock().unwrap();
                self.service
                    .iter()
                    .enumerate()
                    .map(|v| (v.0, v.1.when()))
                    .map(|(index, time)| {
                        runs.get(&index)
                            .map(|v| subtract_durations(time, Instant::now() - *v))
                            .unwrap_or(Duration::ZERO)
                    })
                    .min()
            };
            sleep(next.unwrap()).await;

            for (index, service) in self.service.iter().enumerate() {
                let next = {
                    let interval_time = service.when();
                    self.runs
                        .lock()
                        .unwrap()
                        .get(&index)
                        .map(|v| subtract_durations(interval_time, Instant::now() - *v))
                        .unwrap_or_default()
                };
                if next > Duration::ZERO {
                    continue;
                }
                let started = Instant::now();
                service.run().await;
                self.runs.lock().unwrap().insert(index, started);
            }
        }
    }
}
