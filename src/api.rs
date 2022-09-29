use tiny_http::{Response, Server};

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
            self.api_end
                .api_req_sender
                .send(Message::GetScreentimeReq)
                .unwrap();

            match self
                .api_end
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
}
