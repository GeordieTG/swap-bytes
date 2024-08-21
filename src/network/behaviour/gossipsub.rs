use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, QueryId}, Swarm};
use log::info;

use crate::{network::network::ChatBehaviour, state::STATE};

pub async fn handle_event(event: libp2p::gossipsub::Event, rating_fetch_queue: &mut HashMap<QueryId, (String, String, String)>, swarm: &mut Swarm<ChatBehaviour>) {

    match event {

        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _id,
            message,
        } => {

            let topic = message.topic.to_string();

            log::info!("Received message: {} on Topic: {}", String::from_utf8_lossy(&message.data), topic.clone());
            
            let data = String::from_utf8_lossy(&message.data).to_string();
            let state = STATE.lock().unwrap();
            let nickname = state.nicknames.get(&peer_id.to_string()).expect("User not found").clone();            
            drop(state);

            log::info!("made it");

             // Fetch the users nickname from the DHT
             let key_string = "rating_".to_string() + &peer_id.to_string();
             let key = kad::RecordKey::new(&key_string);
             let query_id = swarm.behaviour_mut().kademlia.get_record(key);
             rating_fetch_queue.insert(query_id, (data, nickname, topic));
        }  

        _ => {}
    }
}