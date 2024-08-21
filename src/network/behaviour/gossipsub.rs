use libp2p::gossipsub;

use crate::state::STATE;

pub async fn handle_event(event: libp2p::gossipsub::Event) {

    match event {

        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _id,
            message,
        } => {

            let topic = message.topic.to_string();

            log::info!("Received message: {} on Topic: {}", String::from_utf8_lossy(&message.data), topic.clone());
            
            let message = String::from_utf8_lossy(&message.data).to_string();

            let mut state = STATE.lock().unwrap();
            
            let nickname = state.nicknames.get(&peer_id.to_string()).expect("User not found").clone();

            if topic == "global".to_string() {
                state.messages.lock().unwrap().push(format!("{}: {}", nickname, message));
            } else {
                state.room_chats.get_mut(&topic.to_string()).expect("").push(format!("{}: {}", nickname, message));
            }
        }  

        _ => {}
    }
}