use chrono::{DateTime, TimeZone, Utc};
use sqlite::{Connection, State};

use crate::structs::{DaemonEnd, Entry, EntryEvent, Message};

const CREATE_TABLE_STATEMENT: &str =
    "CREATE TABLE IF NOT EXISTS screentime (timestamp INTEGER, event_type INTEGER)";
const SELECT_MOST_RECENT_SCREENTIME_ENTRY_STATEMENT: &str =
    "SELECT timestamp, event_type FROM screentime ORDER BY timestamp DESC LIMIT 1";
const SELECT_OLDEST_SCREENTIME_ENTRY_STATEMENT: &str =
    "SELECT timestamp, event_type FROM screentime ORDER BY timestamp LIMIT 1";

const INSERT_SCREENTIME_STATEMENT: &str =
    "INSERT INTO screentime (timestamp, event_type) VALUES (?, ?)";

pub struct Daemon {
    connection: Connection,
    daemon_end: DaemonEnd,
    now: DateTime<Utc>,
    last_activity_time: DateTime<Utc>,
}

impl Daemon {
    pub fn new(connection: Connection, daemon_end: DaemonEnd) -> Self {
        Self {
            connection,
            daemon_end,
            now: Utc::now(),
            last_activity_time: Utc::now(),
        }
    }
    fn insert_now(&mut self) {
        self.connection
            .prepare(INSERT_SCREENTIME_STATEMENT)
            .unwrap()
            .bind(1, self.now.timestamp())
            .unwrap()
            .bind(2, 1)
            .unwrap()
            .next()
            .unwrap();
    }
    fn handle_api_requests(&mut self) {
        let request = self.daemon_end.api_req_receiver.try_recv();

        if request.is_ok() {
            let _request = request.unwrap();
            let last_dt = self.get_oldest_screentime();
            let diff = self.now - last_dt.datetime;
            self.daemon_end
                .api_resp_sender
                .send(Message::GetScreentimeResp(diff.num_seconds()))
                .unwrap();
        }
    }

    fn insert_afk_if_needed(&mut self) {
        let minutes_from_last_activity = (self.now - self.last_activity_time).num_minutes();

        if minutes_from_last_activity < 5 {
            return;
        }

        if self.get_most_recent_screentime().event_type == EntryEvent::Activity {
            return;
        }

        self.connection
            .prepare(INSERT_SCREENTIME_STATEMENT)
            .unwrap()
            .bind(1, self.now.timestamp())
            .unwrap()
            .bind(2, 2)
            .unwrap()
            .next()
            .unwrap();
    }

    pub fn run(&mut self) {
        self.connection.execute(CREATE_TABLE_STATEMENT).unwrap();
        self.insert_now();

        self.last_activity_time = Utc::now();

        loop {
            self.now = Utc::now();

            let event = self.daemon_end.input_events.try_recv();

            self.handle_api_requests();

            self.insert_afk_if_needed();

            match event {
                Err(_) => (),
                Ok(_) => {
                    self.last_activity_time = self.now;
                }
            }
        }
    }

    fn get_most_recent_screentime(&mut self) -> Entry {
        let mut statement = self
            .connection
            .prepare(SELECT_MOST_RECENT_SCREENTIME_ENTRY_STATEMENT)
            .unwrap();

        let mut datetime = self.now;
        let mut event_type = EntryEvent::Activity;

        while let State::Row = statement.next().unwrap() {
            let timestamp: i64 = statement.read(0).unwrap();
            event_type = EntryEvent::try_from(statement.read::<i64>(1).unwrap()).unwrap();
            datetime = Utc.timestamp(timestamp, 0);
            break;
        }

        Entry {
            datetime,
            event_type,
        }
    }

    fn get_oldest_screentime(&mut self) -> Entry {
        let mut statement = self
            .connection
            .prepare(SELECT_OLDEST_SCREENTIME_ENTRY_STATEMENT)
            .unwrap();

        let mut datetime = Utc::now();
        let mut event_type = EntryEvent::Activity;

        while let State::Row = statement.next().unwrap() {
            let timestamp: i64 = statement.read(0).unwrap();
            event_type = EntryEvent::try_from(statement.read::<i64>(1).unwrap()).unwrap();
            datetime = Utc.timestamp(timestamp, 0);
            break;
        }

        Entry {
            datetime,
            event_type,
        }
    }
}
