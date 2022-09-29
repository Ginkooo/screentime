use chrono::{DateTime, Utc};
use crossbeam::channel::{Receiver, Sender};

pub enum InputEvent {
    Unknown,
}

#[derive(PartialEq, Eq)]
pub enum EntryEvent {
    Activity,
    Afk,
    Unknown,
}

impl From<i64> for EntryEvent {
    fn from(num: i64) -> Self {
        match num {
            1 => EntryEvent::Activity,
            2 => EntryEvent::Afk,
            _ => EntryEvent::Unknown,
        }
    }
}

pub enum Message {
    Input(InputEvent),
    GetScreentimeReq,
    GetScreentimeResp(i64),
}

pub struct Entry {
    pub datetime: DateTime<Utc>,
    pub event_type: EntryEvent,
}

#[derive(Clone)]
pub struct InputEventEnd {
    pub input_events: Sender<Message>,
}

pub struct DaemonEnd {
    pub input_events: Receiver<Message>,
    pub api_req_receiver: Receiver<Message>,
    pub api_resp_sender: Sender<Message>,
}

pub struct ApiEnd {
    pub api_req_sender: Sender<Message>,
    pub api_resp_receiver: Receiver<Message>,
}
