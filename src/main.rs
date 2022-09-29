mod daemon;
mod structs;
use structs::{ApiEnd, DaemonEnd, InputEvent, InputEventEnd, Message};
use tiny_http::{Response, Server};

use crossbeam::channel::{unbounded, Receiver, Sender};

use chrono::{DateTime, Duration, TimeZone, Utc};

use rdev::{listen, Event};
use sqlite::{Connection, State};

fn run_api(api_end: ApiEnd) {
    let server = Server::http("127.0.0.1:9898").unwrap();
    for request in server.incoming_requests() {
        dbg!(request.url());
        api_end
            .api_req_sender
            .send(Message::GetScreentimeReq)
            .unwrap();

        match api_end
            .api_resp_receiver
            .recv_timeout(std::time::Duration::from_secs(120))
        {
            Ok(Message::GetScreentimeResp(secs)) => {
                request
                    .respond(Response::from_string(secs.to_string()))
                    .unwrap();
            }
            _ => {}
        };
    }
}

fn input_callback(_event: Event, sender: InputEventEnd) {
    sender
        .input_events
        .send(Message::Input(InputEvent::Unknown))
        .unwrap();
}

fn run_event_listener(sender: InputEventEnd) {
    listen(move |event| {
        input_callback(event, sender.clone());
    })
    .ok();
}

fn initialize_db() -> Connection {
    let mut cache_dir = dirs::cache_dir().unwrap();
    cache_dir.push("screentime");
    std::fs::create_dir_all(&cache_dir).unwrap();
    let mut sqlite_path = cache_dir.clone();
    sqlite_path.push("db.sqlite");
    sqlite::open(sqlite_path).unwrap()
}

fn main() {
    let connection = initialize_db();
    let (input_events_sender, input_events_receiver) = unbounded();
    let (api_req_sender, api_req_receiver) = unbounded();
    let (api_resp_sender, api_resp_receiver) = unbounded();
    std::thread::scope(move |scope| {
        scope.spawn(|| {
            daemon::run_deamon(
                connection,
                DaemonEnd {
                    input_events: input_events_receiver,
                    api_resp_sender,
                    api_req_receiver,
                },
            )
        });
        scope.spawn(|| {
            run_api(ApiEnd {
                api_resp_receiver,
                api_req_sender,
            })
        });
        scope.spawn(|| {
            run_event_listener(InputEventEnd {
                input_events: input_events_sender,
            })
        });
    });
}

#[cfg(test)]
mod tests {
    use reqwest;

    #[test]
    fn test_works() {
        std::thread::spawn(|| {
            super::main();
        });

        std::thread::sleep(std::time::Duration::from_secs(1));
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("http://127.0.0.1:9898/get_screentime/")
            .send()
            .unwrap();
        let resp = resp.text().unwrap();
        dbg!(resp);
    }
}
