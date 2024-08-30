use std::collections::HashMap;

use libp2p::kad::QueryId;
use libp2p::{kad, mdns, PeerId, Swarm};
use crate::state::STATE;
use crate::network::network::ChatBehaviour;

// Handles all MDNS events that come through the network event loop.
pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<ChatBehaviour>, nickname_fetch_queue: &mut HashMap<QueryId, PeerId>) {

    match event {

        // Handles the connection with a new peer. We first add them to Gossipsub, Kademlia and our local list of peers,
        // before fetching their nickname from the DHT.
        mdns::Event::Discovered(list) => {
            for (peer_id, addr) in list {
                log::info!("Connected with person with id: {peer_id}");
    
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    
                {
                    let mut state = STATE.lock().unwrap();
                    state.peers.push(peer_id);
                }
                
                // Fetch the users nickname from the DHT
                let key_string = "nickname_".to_string() + &peer_id.to_string();
                let key = kad::RecordKey::new(&key_string);
                let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                nickname_fetch_queue.insert(query_id, peer_id);
            }
        }

        // Handles the event that a known peer disconnects.
        mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id)
            }
        }
    }
}