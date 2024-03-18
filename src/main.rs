mod api;
mod client;
mod last_usage_time;
mod screentime_updater;
mod utils;

use axum::Router;
use chrono::Utc;

use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use axum::routing::get;

type ScreenTime = HashMap<String, u32>;

fn build_router() -> Router {
    Router::new()
        .route("/inlinehms", get(api::get_inlinehms))
        .route("/json", get(api::get_json_secs))
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).unwrap_or("missing".into()) != *"-d" {
        client::execute(std::env::args().skip(1).collect());
        return;
    }
    let (rx, tx) = single_value_channel::channel_starting_with(Utc::now());
    tokio::task::spawn(screentime_updater::run_screentime_updater(rx));
    tokio::task::spawn(last_usage_time::run_last_usage_time_updater(tx));

    axum::Server::bind(&SocketAddr::from_str("0.0.0.0:8645").unwrap())
        .serve(build_router().into_make_service())
        .await
        .unwrap();
}
