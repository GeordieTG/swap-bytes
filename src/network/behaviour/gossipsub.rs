use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, QueryId}, Swarm};

use crate::{network::network::ChatBehaviour, state::STATE};

/// Handles all Gossipsub events that come through the network event loop.
pub async fn handle_event(event: libp2p::gossipsub::Event, rating_fetch_queue: &mut HashMap<QueryId, (String, String, String)>, swarm: &mut Swarm<ChatBehaviour>) {

    match event {
        
        // In the event we recieve a message, we add the message to a queue while we wait for the retreival of the rating for the
        // user who sent the message. This message will be displayed on screen after this fetch has complete (see kademlia.rs).
        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _id,
            message,
        } => {
                let mut state = STATE.lock().unwrap();

                // Message display information
                let topic = message.topic.to_string();
                let data = String::from_utf8_lossy(&message.data).to_string();
                let nickname = state.nicknames.get(&peer_id.to_string()).expect("User not found").clone();      

                // Notify we have received a message for this room
                state.notifications.insert(topic.clone(), true);       

                // Fetch the users rating from the DHT (appending the message information to a queue)
                let key_string = "rating_".to_string() + &peer_id.to_string();
                let key = kad::RecordKey::new(&key_string);
                let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                rating_fetch_queue.insert(query_id, (data, nickname, topic.clone()));

                log::info!("Received message: {} on Topic: {}", String::from_utf8_lossy(&message.data), topic);
            }  

        other => {
            log::info!("{:?}", other);
        }
    }
}