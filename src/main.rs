use chrono::Local;

use jammdb::DB;

use std::collections::HashMap;
use std::convert::Infallible;

use std::net::SocketAddr;
use std::path::PathBuf;

use std::time::Duration;

use active_win_pos_rs::get_active_window;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;

fn seconds_to_hms(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) - (hours * 60);
    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
}

type ScreenTime = HashMap<String, u32>;

fn get_config_file_path() -> PathBuf {
    let mut config_file = dirs::config_dir().unwrap();
    config_file.push("screentime.db");
    config_file
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

    let config_file = get_config_file_path();

    let db = DB::open(config_file).unwrap();

    {
        let tx = db.tx(true).unwrap();
        tx.get_or_create_bucket("screentime").unwrap();
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

fn get_inlinehms() -> String {
    let config_file = get_config_file_path();
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
            result.join(" ")
        } else {
            "".to_string()
        }
    } else {
        "no data".to_string()
    }
}

async fn hello(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    fn mk_response(s: String) -> Result<Response<Full<Bytes>>, Infallible> {
        Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
    }
    let result = match request.uri().to_string().as_str() {
        "/inlinehms" => get_inlinehms(),
        _ => "not found".to_string(),
    };
    mk_response(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn(run_usage_time_updater());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
