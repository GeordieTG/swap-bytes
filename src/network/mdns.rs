use libp2p::{kad, mdns, PeerId, Swarm};
use crate::state::{self, STATE};
use super::network::ChatBehaviour;


pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<ChatBehaviour>) {

    match event {

        mdns::Event::Discovered(list) => {
            for (peer_id, addr) in list {
                log::info!("Connected with person with id: {peer_id}");
    
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    
                {
                    let state = STATE.lock().unwrap();
                    let mut peers: std::sync::MutexGuard<Vec<PeerId>> = state.peers.lock().unwrap();
                    peers.push(peer_id)
                }
    
                let state: std::sync::MutexGuard<state::GlobalState> = STATE.lock().unwrap();
    
                // Add your nickname to DHT
                if !state.has_added_nickname {
    
                    log::info!("My nickname is {}", state.nickname);
                    let nickname_bytes = serde_cbor::to_vec(&state.nickname).unwrap();
        
                    let record = kad::Record {
                        key: kad::RecordKey::new(&swarm.local_peer_id().to_string()),
                        value: nickname_bytes,
                        publisher: None,
                        expires: None,
                    };
    
                    swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record locally");
                }
            }
        }


        mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id)
            }
        }
    }
}