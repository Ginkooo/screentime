use std::sync::{Arc, RwLock};

use crate::{
    consts::{SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS},
    types::ThreadSafeUsageTime,
    utils,
};
use chrono::{DateTime, Local, Timelike};
use config::Config;
#[derive(Debug)]

struct ActiveWindow {
    _title: String,
    process_name: String,
}

fn get_active_window() -> Option<ActiveWindow> {
    let window = active_win_pos_rs::get_active_window();
    match window {
        Ok(window) => Some(ActiveWindow {
            _title: window.title.to_lowercase(),
            process_name: window.process_name.to_lowercase(),
        }),
        Err(_) => None,
    }
}

pub fn run_usage_time_updater(
    usage_time: ThreadSafeUsageTime,
    last_input_time: Arc<RwLock<DateTime<Local>>>,
    config: &Config,
) {
    loop {
        let active_window = get_active_window().unwrap_or(ActiveWindow {
            _title: String::from("unknown"),
            process_name: String::from("unknown"),
        });

        std::thread::sleep(std::time::Duration::from_secs(1));
        let last_it = last_input_time.read().unwrap();
        let mut value = usage_time.write().unwrap();

        let current_day_snapshot_file_path = utils::get_current_day_snapshot_file_path();
        if !current_day_snapshot_file_path.exists() {
            *value.get_mut("unknown").unwrap() = 0;
            utils::create_current_day_snapshot_file();
        }

        if (Local::now() - *last_it).num_seconds() > config.get_int(SECONDS_BEFORE_AFK).unwrap() {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
            continue;
        }
        *value.entry(active_window.process_name).or_insert(0) += 1;

        if last_it.second() as u64 % config.get::<u64>(SNAPSHOT_INTERVAL_IN_SECONDS).unwrap() == 0 {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
        }
    }
}
