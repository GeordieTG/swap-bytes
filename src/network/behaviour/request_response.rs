use libp2p::request_response::{self};
use libp2p_request_response::Message;
use crate::state::STATE;
use crate::network::network::{Request, Response};

// Handles all Request-Response events that come through the network event loop.
pub async fn handle_event(event: libp2p::request_response::Event<Request, Response>) {

    match event {

        // In the event we receive a Request-Response message.
        request_response::Event::Message { peer, message } => {

            match message {

                // If we receive a request we add it to our global state and this will be shown in the "Incoming Requests" list on
                // the "Direct Messages" tab.
                Message::Request { request, channel, .. } => {
                    log::info!("Received request: {:?}", request);

                    let mut state = STATE.lock().unwrap();
                    state.requests.push((peer, request.message, channel))
                },

                // If we recieve a response we write the file to our local directory and proceed to rate the peer.
                Message::Response { response, .. } => {
                    log::info!("Received response: {:?}", response);

                    if let Err(e) = std::fs::write(&response.filename, response.data) {
                        log::error!("Failed to write file {}: {}", &response.filename, e);
                    } else {
                        log::info!("File {} received and saved successfully", &response.filename);
                    }

                    let mut state = STATE.lock().unwrap();
                    state.current_rating = Some(peer);
                    // state.tab = 3;
                },
            }
        }
        
        other => {
            log::info!("Unhandled {:?}", other);
        }
    }
}