use std::sync::{Arc, RwLock};

use crate::{
    consts::{DESKTOP, SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS},
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
    let mut last_iteration_time = Local::now();
    loop {
        let time_diff_from_last_iteration_in_seconds =
            (Local::now() - last_iteration_time).num_seconds();
        std::thread::sleep(std::time::Duration::from_millis(100));
        if time_diff_from_last_iteration_in_seconds < 1 {
            continue;
        }
        last_iteration_time = Local::now();
        let active_window = get_active_window().unwrap_or(ActiveWindow {
            _title: DESKTOP.clone().to_string(),
            process_name: DESKTOP.clone().to_string(),
        });

        let last_input_time = last_input_time.read().unwrap();
        let mut value = usage_time.write().unwrap();

        let current_day_snapshot_file_path = utils::get_current_day_snapshot_file_path();
        if !current_day_snapshot_file_path.exists() {
            utils::create_current_day_snapshot_file();
        }

        if (Local::now() - *last_input_time).num_seconds()
            > config.get_int(SECONDS_BEFORE_AFK).unwrap()
        {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
            continue;
        }
        *value.entry(active_window.process_name).or_insert(0) +=
            time_diff_from_last_iteration_in_seconds as u64;

        if last_input_time.second() as u64
            % config.get::<u64>(SNAPSHOT_INTERVAL_IN_SECONDS).unwrap()
            == 0
        {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
        }
    }
}
