use std::time::Duration;
use tiny_http::{Response, Server};

use crossbeam::channel::{unbounded, Receiver, Sender};

use chrono::{DateTime, TimeZone, Utc};

use rdev::{listen, Event};
use sqlite::{Connection, State};

enum InputEvent {
    Unknown,
}

enum Message {
    Input(InputEvent),
    GetScreentimeReq,
    GetScreentimeResp(i64),
}

struct Entry {
    datetime: DateTime<Utc>,
    event_type: InputEvent,
}

#[derive(Clone)]
struct InputEventEnd {
    input_events: Sender<Message>,
}

struct DaemonEnd {
    input_events: Receiver<Message>,
    api_req_receiver: Receiver<Message>,
    api_resp_sender: Sender<Message>,
}

struct ApiEnd {
    api_req_sender: Sender<Message>,
    api_resp_receiver: Receiver<Message>,
}

fn insert_now(connection: &Connection) {
    let now = Utc::now();

    connection
        .prepare(INSERT_SCREENTIME_STATEMENT)
        .unwrap()
        .bind(1, now.timestamp())
        .unwrap()
        .bind(2, 1)
        .unwrap()
        .next()
        .unwrap();
}

fn get_most_recent_screentime(connection: &Connection) -> Entry {
    let mut statement = connection
        .prepare(SELECT_MOST_RECENT_SCREENTIME_ENTRY_STATEMENT)
        .unwrap();

    let mut datetime = Utc::now();

    while let State::Row = statement.next().unwrap() {
        let timestamp: i64 = statement.read(0).unwrap();
        let _event_type: i64 = statement.read(1).unwrap();
        datetime = Utc.timestamp(timestamp, 0);
        break;
    }

    Entry {
        datetime,
        event_type: InputEvent::Unknown,
    }
}

const CREATE_TABLE_STATEMENT: &str =
    "CREATE TABLE IF NOT EXISTS screentime (timestamp INTEGER, event_type INTEGER)";
const SELECT_MOST_RECENT_SCREENTIME_ENTRY_STATEMENT: &str =
    "SELECT timestamp, event_type FROM screentime ORDER BY timestamp DESC LIMIT 1";
const INSERT_SCREENTIME_STATEMENT: &str =
    "INSERT INTO screentime (timestamp, event_type) VALUES (?, ?)";

fn run_deamon(connection: Connection, receiver: DaemonEnd) {
    connection.execute(CREATE_TABLE_STATEMENT).unwrap();
    let mut now = Utc::now();
    insert_now(&connection);

    loop {
        now = Utc::now();
        let event = receiver.input_events.try_recv();

        let request = receiver.api_req_receiver.try_recv();

        if request.is_ok() {
            let _request = request.unwrap();
            let last_dt = get_most_recent_screentime(&connection);
            let diff = now - last_dt.datetime;
            receiver
                .api_resp_sender
                .send(Message::GetScreentimeResp(diff.num_minutes()))
                .unwrap();
        }

        match event {
            Err(_) => (),
            Ok(_) => {
                let last_dt = get_most_recent_screentime(&connection).datetime;
                let diff = Utc::now() - last_dt;
                if diff.num_minutes() > 5 {
                    insert_now(&connection)
                }
            }
        }
    }
}

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
            .recv_timeout(Duration::from_secs(2))
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
            run_deamon(
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
    use std::time::Duration;

    #[test]
    fn test_works() {
        std::thread::spawn(|| {
            super::main();
        });

        std::thread::sleep(Duration::from_secs(1));
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("http://127.0.0.1:9898/get_screentime/")
            .send()
            .unwrap();
        let resp = resp.text().unwrap();
        dbg!(resp);
    }
}
