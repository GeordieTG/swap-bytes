use libp2p::request_response::{self};
use libp2p_request_response::Message;
use crate::state::STATE;
use super::network::{Request, Response};

pub async fn handle_event(event: libp2p::request_response::Event<Request, Response>) {

    match event {

        request_response::Event::InboundFailure { error, ..} => {
            log::info!("Inbound Error {error}")
        }

        request_response::Event::OutboundFailure { error, ..} => {
            log::info!("outbound failiure {error}");
        }

        request_response::Event::Message { peer, message } => {

            match message {

                Message::Request { request, channel, .. } => {
                    log::info!("Received request: {:?}", request);
                    let mut state = STATE.lock().unwrap();
                    state.requests.push((peer, request.request, channel))
                },

                Message::Response { response, .. } => {
                    log::info!("Received response: {:?}", response);

                    if let Err(e) = std::fs::write(&response.filename, response.data) {
                        log::error!("Failed to write file {}: {}", &response.filename, e);
                    } else {
                        log::info!("File {} received and saved successfully", &response.filename);
                    }

                    let mut state = STATE.lock().unwrap();
                    state.tab = 3;
                },
            }
        }
        
        _ => {}
    }
}