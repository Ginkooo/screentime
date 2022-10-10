use config::Config;
use tiny_http::{Response, Server};

use crate::types::ThreadSafeUsageTime;

pub fn run_server(usage_time: ThreadSafeUsageTime, config: &Config) {
    let server = Server::http(format!("127.0.0.1:{}", config.get_int("port").unwrap())).unwrap();
    for request in server.incoming_requests() {
        let value = usage_time.read().unwrap();
        let string = value.get("unknown").unwrap().to_string();
        request.respond(Response::from_string(string)).unwrap();
    }
}
