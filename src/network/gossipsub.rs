use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, QueryId}, PeerId, Swarm};

use super::network::ChatBehaviour;

pub async fn handle_event(event: libp2p::gossipsub::Event, swarm: &mut Swarm<ChatBehaviour>, nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String)>) {

    match event {

        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _id,
            message,
        } => {

            log::info!("Received message: {}", String::from_utf8_lossy(&message.data));

            let message = String::from_utf8_lossy(&message.data).to_string();

            // Fetch the users nickname from the DHT
            let key = kad::RecordKey::new(&peer_id.to_string());
            let query_id = swarm.behaviour_mut().kademlia.get_record(key);
            nickname_fetch_queue.insert(query_id, (peer_id, message));
        }  

        _ => {}
    }
}