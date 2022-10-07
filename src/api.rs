use std::sync::{Arc, RwLock};

use config::Config;
use tiny_http::{Response, Server};

pub fn run_server(usage_time: Arc<RwLock<u64>>, config: &Config) {
    let server = Server::http(format!("127.0.0.1:{}", config.get_int("port").unwrap())).unwrap();
    for request in server.incoming_requests() {
        let value = usage_time.read().unwrap();
        let string = value.to_string();
        request.respond(Response::from_string(string)).unwrap();
    }
}
