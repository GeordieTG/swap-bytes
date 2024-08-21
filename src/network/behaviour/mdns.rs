use std::collections::HashMap;

use libp2p::kad::QueryId;
use libp2p::{kad, mdns, PeerId, Swarm};
use crate::state::{self, STATE};
use crate::network::network::ChatBehaviour;


pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<ChatBehaviour>, nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String, String)>) {

    match event {

        mdns::Event::Discovered(list) => {
            for (peer_id, addr) in list {
                log::info!("Connected with person with id: {peer_id}");
    
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    
                {
                    let state = STATE.lock().unwrap();
                    let mut peers: std::sync::MutexGuard<Vec<PeerId>> = state.peers.lock().unwrap();
                    peers.push(peer_id);
                    drop(peers);
                    drop(state);
                }

                
                // Fetch the users nickname from the DHT
                let key = kad::RecordKey::new(&peer_id.to_string());
                let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                nickname_fetch_queue.insert(query_id, (peer_id, "".to_string(), "".to_string()));
                
            }
        }

        mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id)
            }
        }
    }
}