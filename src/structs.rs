use chrono::{DateTime, Utc};
use crossbeam::channel::{Receiver, Sender};

pub enum InputEvent {
    Unknown,
}

pub enum EntryEvent {
    Activity,
    Afk,
}

impl TryFrom<i64> for EntryEvent {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EntryEvent::Activity),
            2 => Ok(EntryEvent::Afk),
            _ => Err(()),
        }
    }
}

impl Into<i64> for EntryEvent {
    fn into(self) -> i32 {
        match self {
            Self::Activity => 1,
            Self::Afk => 2,
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

impl Try
