use std::path::PathBuf;

use active_win_pos_rs::get_active_window;
use chrono::Local;

pub struct Config {}

impl Config {
    pub fn get_screentime_file_path() -> PathBuf {
        let mut config_file = dirs::data_local_dir().unwrap();
        config_file.push("screentime.db");
        config_file
    }
}

pub fn get_today_as_str() -> String {
    let dt = Local::now();
    let dt = dt.format("%Y-%m-%d");
    dt.to_string()
}

pub fn get_focused_program_name() -> String {
    if let Ok(window) = get_active_window() {
        let process_name = window
            .process_path
            .file_name()
            .unwrap()
            .to_owned()
            .to_ascii_lowercase();

        let title = window.title;
        if title.to_lowercase().starts_with("vim") || title.to_lowercase().starts_with("nvim") {
            title.split(" ").nth(0).unwrap().to_string()
        } else {
            process_name.into_string().unwrap()
        }
    } else {
        "unknown".to_string()
    }
}

pub fn seconds_to_hms(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) - (hours * 60);
    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use crate::utils::seconds_to_hms;

    #[test]
    fn test_seconds_to_hms() {
        assert_eq!(seconds_to_hms(3600), "01:00:00");
        assert_eq!(seconds_to_hms(2400), "00:40:00");
        assert_eq!(seconds_to_hms(4325), "01:12:05");
    }
}
