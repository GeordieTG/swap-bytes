use std::collections::HashMap;

use libp2p::kad::QueryId;
use libp2p::{gossipsub, kad, mdns, PeerId, Swarm};
use crate::state::STATE;
use crate::network::network::ChatBehaviour;
use crate::util;

// Handles all MDNS events that come through the network event loop.
pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<ChatBehaviour>, nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String)>) {

    match event {

        // Handles the connection with a new peer.
        mdns::Event::Discovered(list) => {

            for (peer_id, addr) in list {
                log::info!("Connected with person with id: {peer_id}");
    
                // Add the peer to our network
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    
                // Update local peers
                let mut state = STATE.lock().unwrap();
                state.peers.push(peer_id);
                
                // Subscribe to the DM for this user and add it to our local storage
                let own_peer_id = state.peer_id.clone();
                let dm_key = util::format_dm_key(peer_id.to_string(), own_peer_id);
                let topic = gossipsub::IdentTopic::new(dm_key.clone());
                swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
                
                // Fetch the users nickname from the DHT
                let key_string = "nickname_".to_string() + &peer_id.to_string();
                let key = kad::RecordKey::new(&key_string);
                let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                nickname_fetch_queue.insert(query_id, (peer_id, dm_key));
            }
        }

        _ => {}
    }
}