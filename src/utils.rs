use std::{fs::File, io::Write, path::PathBuf};

use chrono::Local;

use crate::types::UsageTime;

pub fn get_current_day_snapshot_file_path() -> PathBuf {
    let mut path = get_cache_dir_file_path();
    let today_date_str = Local::now().date_naive().to_string();
    path.push(today_date_str);
    path
}

pub fn get_cache_dir_file_path() -> PathBuf {
    let mut snapshot_file_path = dirs::cache_dir().unwrap();
    snapshot_file_path.push("screentime");
    snapshot_file_path
}

pub fn create_cache_dir() {
    std::fs::create_dir_all(get_cache_dir_file_path()).unwrap();
}

pub fn create_current_day_snapshot_file() -> Result<File, Box<dyn std::error::Error>> {
    let path = get_current_day_snapshot_file_path();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .expect("could not open current day file");
    file.write_all(serde_json::to_string_pretty(&UsageTime::new())?.as_bytes())?;
    Ok(file)
}

pub fn write_usage_time_to_file(value: &UsageTime, path: &PathBuf) {
    let bytes = serde_json::to_string_pretty(&value).unwrap();
    std::fs::write(path, bytes).unwrap();
}
