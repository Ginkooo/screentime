use chrono::{DateTime, TimeZone, Utc};
use sqlite::{Connection, State};

use crate::structs::{DaemonEnd, Entry, EntryEvent, InputEvent, Message};

const CREATE_TABLE_STATEMENT: &str =
    "CREATE TABLE IF NOT EXISTS screentime (timestamp INTEGER, event_type INTEGER)";
const SELECT_MOST_RECENT_SCREENTIME_ENTRY_STATEMENT: &str =
    "SELECT timestamp, event_type FROM screentime ORDER BY timestamp DESC LIMIT 1";
const SELECT_OLDEST_SCREENTIME_ENTRY_STATEMENT: &str =
    "SELECT timestamp, event_type FROM screentime ORDER BY timestamp LIMIT 1";

const INSERT_SCREENTIME_STATEMENT: &str =
    "INSERT INTO screentime (timestamp, event_type) VALUES (?, ?)";
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
    let mut event_type = EntryEvent::Activity;

    while let State::Row = statement.next().unwrap() {
        let timestamp: i64 = statement.read(0).unwrap();
        event_type = EntryEvent::try_from(statement.read(1).unwrap()).unwrap();
        datetime = Utc.timestamp(timestamp, 0);
        break;
    }

    Entry {
        datetime,
        event_type,
    }
}

fn get_oldest_screentime(connection: &Connection) -> Entry {
    let mut statement = connection
        .prepare(SELECT_OLDEST_SCREENTIME_ENTRY_STATEMENT)
        .unwrap();

    let mut datetime = Utc::now();
    let mut event_type = EntryEvent::Activity;

    while let State::Row = statement.next().unwrap() {
        let timestamp: i64 = statement.read(0).unwrap();
        event_type = EntryEvent::try_from(statement.read(1).unwrap()).unwrap();
        datetime = Utc.timestamp(timestamp, 0);
        break;
    }

    Entry {
        datetime,
        event_type: InputEvent::Unknown,
    }
}

fn insert_afk_if_needed(connection: &Connection, last_activity_time: DateTime<Utc>) {
    let now = Utc::now();
    let minutes_from_last_activity = (now - last_activity_time).num_minutes();

    if minutes_from_last_activity < 5 {
        return;
    }

    if get_most_recent_screentime(&connection).event_type == EntryEvent::Activity {
        return;
    }

    connection
        .prepare(INSERT_SCREENTIME_STATEMENT)
        .unwrap()
        .bind(1, now.timestamp())
        .unwrap()
        .bind(2, 2)
        .unwrap()
        .next()
        .unwrap();
}

fn handle_api_requests(receiver: &mut DaemonEnd, connection: &Connection) {
    let now = Utc::now();
    let request = receiver.api_req_receiver.try_recv();

    if request.is_ok() {
        let _request = request.unwrap();
        let last_dt = get_oldest_screentime(&connection);
        let diff = now - last_dt.datetime;
        receiver
            .api_resp_sender
            .send(Message::GetScreentimeResp(diff.num_seconds()))
            .unwrap();
    }
}

pub fn run_deamon(connection: Connection, receiver: DaemonEnd) {
    connection.execute(CREATE_TABLE_STATEMENT).unwrap();
    insert_now(&connection);

    let mut last_activity_time = Utc::now();

    loop {
        let now = Utc::now();

        let event = receiver.input_events.try_recv();

        handle_api_requests(&mut receiver, &connection);

        insert_afk_if_needed(&connection, last_activity_time);

        match event {
            Err(_) => (),
            Ok(_) => {
                last_activity_time = now;
            }
        }
    }
}
