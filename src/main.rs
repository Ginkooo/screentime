use chrono::{DateTime, Local};
use rdev::listen;
use std::{
    fs::{File},
    io::Write,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tiny_http::{Response, Server};

fn write_usage_time_to_file(value: u64, path: &PathBuf) {
    std::fs::write(path, &value.to_string().into_bytes()[..]).unwrap();
}

const SECONDS_BEFORE_AFK: u32 = 30;
const SNAPSHOT_INTERVAL_IN_SECONDS: u32 = 10;

fn get_current_day_snapshot_file_path() -> PathBuf {
    let mut path = get_cache_dir_file_path();
    let today_date_str = Local::now().date_naive().to_string();
    path.push(today_date_str);
    path
}

fn get_cache_dir_file_path() -> PathBuf {
    let mut snapshot_file_path = dirs::cache_dir().unwrap();
    snapshot_file_path.push("screentime");
    snapshot_file_path
}

fn create_cache_dir() {
    std::fs::create_dir_all(get_cache_dir_file_path()).unwrap();
}

fn create_current_day_snapshot_file() -> File {
    let path = get_current_day_snapshot_file_path();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .expect("could not open current day file");
    file.write(&[0]).unwrap();
    file
}

fn run_server(usage_time: Arc<RwLock<u64>>) {
    let server = Server::http("127.0.0.1:9898").unwrap();
    for request in server.incoming_requests() {
        let value = usage_time.read().unwrap();
        let string = value.to_string();
        request.respond(Response::from_string(string)).unwrap();
    }
}

fn run_input_event_listener(last_input_time: Arc<RwLock<DateTime<Local>>>) {
    listen(move |_| {
        let last = &last_input_time;
        let mut value = last.write().unwrap();
        *value = Local::now();
    })
    .unwrap();
}

fn run_usage_time_updater(
    usage_time: Arc<RwLock<u64>>,
    last_input_time: Arc<RwLock<DateTime<Local>>>,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let last_it = last_input_time.read().unwrap();
        let mut value = usage_time.write().unwrap();

        let current_day_snapshot_file_path = get_current_day_snapshot_file_path();
        if !current_day_snapshot_file_path.exists() {
            *value = 0;
            create_current_day_snapshot_file();
        }

        if (Local::now() - *last_it).num_seconds() > SECONDS_BEFORE_AFK.into() {
            write_usage_time_to_file(*value, &get_current_day_snapshot_file_path());
            continue;
        }
        *value += 1;
        if *value % SNAPSHOT_INTERVAL_IN_SECONDS as u64 == 0 {
            write_usage_time_to_file(*value, &get_current_day_snapshot_file_path());
        }
    }
}

fn main() {
    create_cache_dir();
    let snapshot_file_path = get_current_day_snapshot_file_path();

    let bytes = std::fs::read(&snapshot_file_path);
    let mut usage_time = 0u64;
    match bytes {
        Ok(bytes) => {
            usage_time = String::from_utf8(bytes)
                .unwrap_or("0".to_string())
                .parse()
                .unwrap()
        }
        Err(_) => {}
    }

    let usage_time = Arc::new(RwLock::new(usage_time));
    let last_input_time = Arc::new(RwLock::new(Local::now()));
    let last_input_time_clone1 = last_input_time.clone();
    let usage_time_clone_1 = usage_time.clone();

    std::thread::scope(|scope| {
        scope.spawn(|| run_usage_time_updater(usage_time_clone_1, last_input_time_clone1));
        scope.spawn(|| run_server(usage_time));
        scope.spawn(|| run_input_event_listener(last_input_time));
    });
}
