use tiny_http::{Request, Response, Server};

use crate::structs::{ApiEnd, Message};

pub struct Api {
    api_end: ApiEnd,
}

impl Api {
    pub fn new(api_end: ApiEnd) -> Self {
        Self { api_end }
    }

    pub fn run(&self) {
        let server = Server::http("127.0.0.1:9898").unwrap();
        for request in server.incoming_requests() {
            dbg!(request.url());
            let chars_to_strip = &['/', ' '][..];
            let url = request.url().trim_matches(chars_to_strip);

            self.send_request_to_channel(url);
            self.send_response_if_some(request);
        }
    }

    fn send_request_to_channel(&self, url: &str) {
        let message = match url {
            "get_total_screentime" => Some(Message::GetScreentimeReq),
            _ => None,
        };

        if let Some(message) = message {
            self.api_end.api_req_sender.send(message).unwrap();
        }
    }

    fn send_response_for_message(&self, request: Request, message: Message) {
        let response = match message {
            Message::GetScreentimeResp(secs) => Some(Response::from_string(secs.to_string())),
            _ => None,
        };

        if let Some(response) = response {
            request.respond(response).unwrap();
        }
    }

    fn send_response_if_some(&self, request: Request) {
        match self
            .api_end
            .api_resp_receiver
            .recv_timeout(std::time::Duration::from_secs(120))
        {
            Ok(message) => self.send_response_for_message(request, message),
            Err(_) => {}
        };
    }
}
