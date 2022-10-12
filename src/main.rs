mod api;
mod client;
mod consts;
mod daemon;
mod types;
mod utils;

use chrono::{DateTime, Local};
use config::Config;
use consts::{
    DEFAULT_PORT, DEFAULT_SECONDS_BEFORE_AFK, DEFAULT_SNAPSHOT_INTERVAL_IN_SECONDS, PORT,
    SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS,
};
use rdev::listen;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use types::{ThreadSafeUsageTime, UsageTime};

fn run_input_event_listener(last_input_time: Arc<RwLock<DateTime<Local>>>) {
    listen(move |_| {
        let last = &last_input_time;
        let mut value = last.write().unwrap();
        *value = Local::now();
    })
    .unwrap();
}

fn build_config() -> Config {
    Config::builder()
        .add_source(config::File::with_name(
            utils::get_created_config_file_path().to_str().unwrap(),
        ))
        .set_default(PORT, DEFAULT_PORT)
        .unwrap()
        .set_default(
            SNAPSHOT_INTERVAL_IN_SECONDS,
            DEFAULT_SNAPSHOT_INTERVAL_IN_SECONDS,
        )
        .unwrap()
        .set_default(SECONDS_BEFORE_AFK, DEFAULT_SECONDS_BEFORE_AFK)
        .unwrap()
        .build()
        .unwrap()
}

fn main() {
    let config = build_config();

    let arg_list = std::env::args().skip(1);
    if arg_list.len() == 1 {
        let option = &arg_list.collect::<Vec<String>>()[0];
        client::handle_client_mode(option, &config);
        return;
    }

    utils::create_cache_dir();
    let snapshot_file_path = utils::get_current_day_snapshot_file_path();

    let bytes = std::fs::read(&snapshot_file_path);
    let mut usage_time = UsageTime::new();

    if let Ok(bytes) = bytes {
        let dupa = String::from_utf8(bytes).expect("corrupted screentime file");
        match serde_json::from_str(dupa.as_str()) {
            Ok(value) => usage_time = value,
            Err(_) => {
                usage_time = UsageTime::new();
            }
        }
    }

    let usage_time: ThreadSafeUsageTime = Arc::new(RwLock::new(usage_time));
    let last_input_time = Arc::new(RwLock::new(Local::now()));
    let last_input_time_clone1 = last_input_time.clone();
    let usage_time_clone_1 = usage_time.clone();

    std::thread::scope(|scope| {
        scope.spawn(|| {
            daemon::run_usage_time_updater(usage_time_clone_1, last_input_time_clone1, &config)
        });
        scope.spawn(|| api::run_server(usage_time, &config));
        scope.spawn(|| run_input_event_listener(last_input_time));
    });
}
