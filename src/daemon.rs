use std::sync::{Arc, RwLock};

use crate::{
    consts::{SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS},
    utils,
};
use chrono::{DateTime, Local};
use config::Config;

pub fn run_usage_time_updater(
    usage_time: Arc<RwLock<u64>>,
    last_input_time: Arc<RwLock<DateTime<Local>>>,
    config: &Config,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let last_it = last_input_time.read().unwrap();
        let mut value = usage_time.write().unwrap();

        let current_day_snapshot_file_path = utils::get_current_day_snapshot_file_path();
        if !current_day_snapshot_file_path.exists() {
            *value = 0;
            utils::create_current_day_snapshot_file();
        }

        if (Local::now() - *last_it).num_seconds() > config.get_int(SECONDS_BEFORE_AFK).unwrap() {
            utils::write_usage_time_to_file(*value, &utils::get_current_day_snapshot_file_path());
            continue;
        }
        *value += 1;
        if *value % config.get::<u64>(SNAPSHOT_INTERVAL_IN_SECONDS).unwrap() == 0 {
            utils::write_usage_time_to_file(*value, &utils::get_current_day_snapshot_file_path());
        }
    }
}
