use api_structure::now_timestamp;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

static mut RERUN: u128 = 0;

#[allow(dead_code)]
//TODO: refactor
pub async fn internal_service(_: impl Fn() -> Arc<Surreal<Db>>) {
    loop {
        let time = get_next_rerun();

        if time > now_timestamp().expect("time went backwards").as_millis() {
            continue;
        }
        debug!("Run Internal Service");
        //todo semi important: register here
        let next = (now_timestamp().expect("time went backwards") + Duration::from_secs(60 * 60))
            .as_millis(); //run every hour
        let next_alt = get_next_rerun();
        if next_alt > time && next_alt < next {
            set_rerun(next_alt)
        } else {
            set_rerun(next)
        }
        debug!("Done");
    }
}

#[allow(dead_code)]
//TODO: refactror
pub fn should_rerun() {
    set_rerun(now_timestamp().expect("time went backwards").as_millis() - 1)
}

#[allow(dead_code)]
fn get_next_rerun() -> u128 {
    unsafe { RERUN }
}

#[allow(dead_code)]
fn set_rerun(i: u128) {
    unsafe { RERUN = i }
}
