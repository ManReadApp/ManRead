use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    sync::{Mutex, Notify},
    time::sleep,
};

use crate::event::{Event, RunningInfo};
#[derive(Default, Clone)]
pub struct EventStore {
    events: Arc<Mutex<Vec<EventWrapper>>>,
    notify: Arc<Notify>,
    rs: Arc<Mutex<RunState>>,
    next_id: Arc<AtomicU64>,
}

struct EventWrapper {
    event_id: u64,
    event: Box<dyn Event + Send>,
}

impl Deref for EventWrapper {
    type Target = Box<dyn Event + Send>;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
#[derive(Default)]
struct RunState {
    last_run: HashMap<u64, Duration>,
}

impl EventStore {
    pub async fn add(&self, event: Box<dyn Event>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.events.lock().await.push(EventWrapper {
            event,
            event_id: id,
        });
        self.notify.notify_one();
    }
    pub fn start_loop(&self) {
        let s = self.clone();
        tokio::spawn(async move {
            s.notify.notify_one();
            let mut remove = Vec::new();
            loop {
                s.notify.notified().await;
                //TODO: drain notifies, to reduce load
                let mut events = s.events.lock().await;
                let mut running = RunningInfo::default();
                for item in events.iter() {
                    if item.is_running() {
                        running.extend(item.as_ref());
                    }
                }
                for item in events.iter() {
                    if item.is_running() || item.needs_wait(&running) {
                        continue;
                    }
                    let mut rs = s.rs.lock().await;
                    let last_run = rs.last_run.get(&item.event_id).copied();
                    let rerun_schedule = item.rerun();
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    match (last_run, rerun_schedule) {
                        (None, _) => { /*Hasnt run yet*/ }
                        (Some(_), None) => {
                            remove.push(item.event_id);
                            continue;
                        }
                        (Some(last_run), Some(wait_duration)) => {
                            let next_run = last_run + wait_duration;
                            if next_run > now {
                                let sleep_dur = next_run - now;
                                let no = s.notify.clone();
                                tokio::spawn(async move {
                                    sleep(sleep_dur).await;
                                    no.notify_one();
                                });
                                continue;
                            }
                        }
                    }
                    rs.last_run.insert(item.event_id, now);
                    item.spawn_execute(s.notify.clone());

                    running.extend(item.as_ref());
                }
                events.retain(|item| !remove.contains(&item.event_id));
                remove.drain(..);
            }
        });
    }
}
