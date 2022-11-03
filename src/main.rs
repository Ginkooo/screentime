mod api;
mod client;
mod config;
mod consts;
mod daemon;
mod types;
mod utils;

use crate::config::ScreentimeConfig;
use chrono::{DateTime, Local};
use clap::{Parser, ValueEnum};
use rdev::listen;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};
use types::{ThreadSafeUsageTime, UsageTime};

/// A screentime monitoring tool. Firstly, start this program with no arguments (daemon mode)
#[derive(Parser)]
struct Args {
    /// Client commands
    #[arg(value_enum)]
    command: Option<Command>,

    /// Specify a config path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(ValueEnum, Clone)]
pub enum Command {
    /// Print total screentime in HH:MM:SS format
    Hms,
    /// Print total screentime in seconds
    Total,
    /// Print a nice-looking summary
    Summary,
    /// Print a summary in raw JSON
    RawSummary,
}

fn run_input_event_listener(last_input_time: Arc<RwLock<DateTime<Local>>>) {
    loop {
        let last_input_time_clone = last_input_time.clone();
        if let Err(err) = listen(move |_| {
            let last = &last_input_time_clone;
            let mut value = last.write().unwrap();
            *value = Local::now();
        }) {
            eprintln!("{:?}. Retry in a sec...", err);
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn main() {
    let args = Args::parse();
    let config = if let Some(config_path) = args.config {
        ScreentimeConfig::new(config_path)
    } else {
        ScreentimeConfig::default()
    };

    if let Some(command) = args.command {
        client::handle_client_mode(command, &config);
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
