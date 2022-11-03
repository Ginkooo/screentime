use std::collections::HashMap;

use colored::Colorize;

use tiny_http::{Request, Response, Server};

use crate::{
    types::{ThreadSafeUsageTime, UsageTime},
    ScreentimeConfig,
};

struct Responder<'a> {
    request: Request,
    value: &'a UsageTime,
    path: String,
}

impl<'a> Responder<'a> {
    fn total(&self) -> Vec<u8> {
        self.value.values().sum::<u64>().to_string().into()
    }

    fn get_hms_usage_time(&self) -> HashMap<String, String> {
        let mut new_hashmap = HashMap::new();
        for (k, &v) in self.value {
            new_hashmap.insert(k.clone(), self.seconds_to_hms(v));
        }
        new_hashmap
    }

    fn raw_summary(&self) -> Vec<u8> {
        let hms_hashmap = self.get_hms_usage_time();
        serde_json::to_string_pretty(&hms_hashmap).unwrap().into()
    }

    fn summary(&self) -> Vec<u8> {
        let hms_hashmap = self.get_hms_usage_time();

        let mut summary = vec![];
        summary.push("Usage Time:\n\n".blue().as_bytes().to_owned());
        for (name, hms) in hms_hashmap {
            summary.push(format!("   {}: {}\n", name, hms).as_bytes().to_owned());
        }

        summary.into_iter().flatten().collect()
    }

    fn seconds_to_hms(&self, seconds: u64) -> String {
        let hours = seconds / 3600;
        let seconds = seconds % 3600;
        let minutes = seconds / 60;
        let seconds = seconds % 60;

        format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
    }

    fn hms(&self) -> Vec<u8> {
        let total_seconds = self.value.values().sum();
        self.seconds_to_hms(total_seconds).into()
    }

    fn respond(self) {
        let body = match self.path.as_str() {
            "total" => self.total(),
            "raw_summary" => self.raw_summary(),
            "hms" => self.hms(),
            "summary" => self.summary(),
            _ => self.total(),
        };

        self.request
            .respond(Response::from_data(body))
            .expect("could not respond");
    }
}

pub fn run_server(usage_time: ThreadSafeUsageTime, config: &ScreentimeConfig) {
    let server = Server::http(format!(
        "127.0.0.1:{}",
        config.config.get_int("port").unwrap()
    ))
    .unwrap();
    for request in server.incoming_requests() {
        let url = request
            .url()
            .split('/')
            .last()
            .unwrap_or("total")
            .to_string();
        let value = usage_time.read().unwrap();

        Responder {
            request,
            value: &*value,
            path: url,
        }
        .respond();
    }
}
