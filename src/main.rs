mod api;
mod daemon;
mod input_event;
mod structs;
use api::Api;
use daemon::Daemon;
use input_event::InputEventListener;
use structs::{ApiEnd, DaemonEnd, InputEvent, InputEventEnd, Message};

use crossbeam::channel::unbounded;

use sqlite::Connection;

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

    let api = Api::new(ApiEnd {
        api_resp_receiver,
        api_req_sender,
    });

    let mut daemon = Daemon::new(
        connection,
        DaemonEnd {
            input_events: input_events_receiver,
            api_resp_sender,
            api_req_receiver,
        },
    );

    let mut input_event_listener = InputEventListener::new(InputEventEnd {
        input_events: input_events_sender,
    });

    std::thread::scope(|scope| {
        scope.spawn(move || daemon.run());
        scope.spawn(move || api.run());
        scope.spawn(|| input_event_listener.run());
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
