use chrono::{DateTime, Local, Timelike, Utc};
use rdev::listen;
use std::{
    ascii::AsciiExt,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tiny_http::{Response, Server};

fn write_usage_time_to_file(value: u64, path: &PathBuf) {
    std::fs::write(path, &value.to_string().into_bytes()[..]).unwrap();
}

fn main() {
    let mut snapshot_file_path = dirs::cache_dir().unwrap();
    snapshot_file_path.push("screentime.txt");

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
    let last_input_time_clone2 = last_input_time.clone();
    let usage_time_clone_1 = usage_time.clone();

    std::thread::scope(|scope| {
        scope.spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let last_it = last_input_time_clone1.read().unwrap();
            let mut value = usage_time_clone_1.write().unwrap();
            let now = Local::now();
            let is_midnight = now.hour() == 0 && now.minute() == 0 && now.second() == 0;
            if is_midnight {
                *value = 0;
            }
            if (Local::now() - *last_it).num_seconds() > 5 {
                write_usage_time_to_file(*value, &snapshot_file_path);
                continue;
            }
            *value += 1;
            if *value % 5 == 0 {
                write_usage_time_to_file(*value, &snapshot_file_path);
            }
        });
        scope.spawn(|| {
            let server = Server::http("127.0.0.1:9898").unwrap();
            for request in server.incoming_requests() {
                let value = usage_time.read().unwrap();
                let string = value.to_string();
                request.respond(Response::from_string(string)).unwrap();
            }
        });
        scope.spawn(|| {
            listen(move |_| {
                let last = &last_input_time_clone2;
                let mut value = last.write().unwrap();
                *value = Local::now();
            })
            .unwrap();
        });
    });
}
