use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, QueryId}, PeerId, Swarm};

use crate::network::network::ChatBehaviour;

pub async fn handle_event(event: libp2p::gossipsub::Event, swarm: &mut Swarm<ChatBehaviour>, nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String, String)>) {

    match event {

        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _id,
            message,
        } => {

            let topic = message.topic.to_string();

            log::info!("Received message: {} on Topic: {}", String::from_utf8_lossy(&message.data), topic.clone());
            
            let message = String::from_utf8_lossy(&message.data).to_string();

            // Fetch the users nickname from the DHT
            let key = kad::RecordKey::new(&peer_id.to_string());
            let query_id = swarm.behaviour_mut().kademlia.get_record(key);
            nickname_fetch_queue.insert(query_id, (peer_id, message, topic));
        }  

        _ => {}
    }
}