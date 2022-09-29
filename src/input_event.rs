use rdev::{listen, Event};

use crate::structs::{InputEvent, InputEventEnd, Message};

pub struct InputEventListener {
    input_event_end: InputEventEnd,
}

impl InputEventListener {
    pub fn new(input_event_end: InputEventEnd) -> Self {
        Self { input_event_end }
    }

    fn input_callback(&mut self, _event: Event) {
        self.input_event_end
            .input_events
            .send(Message::Input(InputEvent::Unknown))
            .unwrap();
    }

    pub fn run(mut self) {
        listen(move |event| {
            self.input_callback(event);
        })
        .ok();
    }
}
