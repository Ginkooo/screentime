use std::sync::{Arc, RwLock};

use crate::{
    consts::{DESKTOP, SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS},
    types::ThreadSafeUsageTime,
    utils, ScreentimeConfig,
};
use chrono::{DateTime, Local, Timelike};
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
    config: &ScreentimeConfig,
) {
    loop {
        let last_iteration_at = Local::now();

        std::thread::sleep(std::time::Duration::from_secs(1));

        let active_window = get_active_window().unwrap_or(ActiveWindow {
            _title: DESKTOP.to_string(),
            process_name: DESKTOP.to_string(),
        });

        let last_input_time = last_input_time.read().unwrap();
        let mut value = usage_time.write().unwrap();

        let current_day_snapshot_file_path = utils::get_current_day_snapshot_file_path();
        if !current_day_snapshot_file_path.exists() {
            utils::create_current_day_snapshot_file().expect("snapshot file must be created");
        }

        if (Local::now() - *last_input_time).num_seconds()
            > config
                .config
                .get_int(SECONDS_BEFORE_AFK)
                .expect("should never return an error, as there is a default value for it")
        {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
            continue;
        }

        let second_passed_since_last_update = (Local::now() - last_iteration_at).num_seconds();
        *value.entry(active_window.process_name).or_insert(0) +=
            second_passed_since_last_update as u64;

        if last_input_time.second() as u64
            % config
                .config
                .get::<u64>(SNAPSHOT_INTERVAL_IN_SECONDS)
                .expect("should never fail, as there is a default value for it")
            == 0
        {
            utils::write_usage_time_to_file(&*value, &utils::get_current_day_snapshot_file_path());
        }
    }
}
