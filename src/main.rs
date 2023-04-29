use axum::Router;
use chrono::Local;

use jammdb::DB;

use mockall::automock;
use std::collections::HashMap;

use axum::routing::get;

use std::path::PathBuf;

use mockall::predicate::*;
use std::time::Duration;

use active_win_pos_rs::get_active_window;

fn seconds_to_hms(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) - (hours * 60);
    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
}

type ScreenTime = HashMap<String, u32>;

pub struct Config {}

#[automock]
impl Config {
    pub fn get_screentime_file_path() -> PathBuf {
        let mut config_file = dirs::data_local_dir().unwrap();
        config_file.push("screentime.db");
        config_file
    }
}

fn get_today_as_str() -> String {
    let dt = Local::now();
    let dt = dt.format("%Y-%m-%d");
    dt.to_string()
}

fn update_screentime() {
    let title = if let Ok(window) = get_active_window() {
        window.process_name.to_lowercase()
    } else {
        "unknown".to_string()
    };

    let config_file = Config::get_screentime_file_path();

    let db = DB::open(config_file).unwrap();

    {
        let tx = db.tx(true).unwrap();
        tx.get_or_create_bucket("screentime").unwrap();
        tx.commit().unwrap();
    }

    let dt = get_today_as_str();
    let mut screentime: ScreenTime;
    {
        let tx = db.tx(false).unwrap();
        let bucket = tx.get_bucket("screentime").unwrap();
        if let Some(data) = bucket.get(&dt) {
            screentime = rmp_serde::from_slice(data.kv().value()).unwrap();
            *screentime.entry(title).or_insert(0) += 1;
        } else {
            screentime = ScreenTime::new();
            *screentime.entry(title).or_insert(0) += 1;
        }
    }
    let tx = db.tx(true).unwrap();
    let bucket = tx.get_or_create_bucket("screentime").unwrap();
    bucket
        .put(dt, rmp_serde::to_vec(&screentime).unwrap())
        .unwrap();
    tx.commit().unwrap();
}

async fn run_usage_time_updater() {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        update_screentime();
    }
}

async fn get_inlinehms() -> String {
    let config_file = Config::get_screentime_file_path();
    let dt = get_today_as_str();
    let db = DB::open(config_file).unwrap();
    let tx = db.tx(false).unwrap();

    if let Ok(bucket) = tx.get_bucket("screentime") {
        if let Some(data) = bucket.get(dt) {
            let screentime: ScreenTime = rmp_serde::from_slice(data.kv().value()).unwrap();
            let mut result: Vec<String> = screentime
                .iter()
                .map(|(k, v)| format!("{}: {}", k, seconds_to_hms(*v)))
                .collect();
            result.sort();
            let total = format!(
                "total: {}",
                seconds_to_hms(screentime.values().sum::<u32>())
            );
            format!("{} {}", total, result.join(" "))
        } else {
            "".to_string()
        }
    } else {
        "no data".to_string()
    }
}

fn build_router() -> Router {
    Router::new().route("/inlinehms", get(get_inlinehms))
}

#[tokio::main]
async fn main() {
    tokio::task::spawn(run_usage_time_updater());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8465").await.unwrap();
    axum::serve(listener, build_router()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::Result;

    use rstest::fixture;
    use rstest::rstest;
    use tempfile::TempDir;

    use crate::MockConfig;

    #[fixture]
    fn screentime_path() -> Result<PathBuf> {
        let tempdir = TempDir::new()?;
        let mut path = tempdir.into_path();
        path.push("screentime");
        Ok(path)
    }

    #[fixture]
    fn mocked_config(_screentime_path: Result<PathBuf>) -> Result<()> {
        let _config = MockConfig::new();
        todo!();
    }

    #[rstest]
    #[tokio::test]
    async fn test_it_works(_screentime_path: Result<PathBuf>, _mocked_config: Result<()>) {
        use axum_test_helper::TestClient;

        let client = TestClient::new(crate::build_router());

        let resp = client.get("/inlinehms").send().await;
        dbg!(resp.text().await);
    }
}
