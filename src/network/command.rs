use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, store::RecordStore, QueryId}, PeerId, Swarm};
use libp2p_request_response::ResponseChannel;

use super::network::{ChatBehaviour, Request, Response};

#[derive(Debug)]
pub enum Command {
    SendMessage {
        message: String,
        room: String
    },
    RequestFile {
        message: String,
        peer: PeerId,
    },
    RespondFile {
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    },
    UpdateRating {
        peer: PeerId,
        rating: i32
    },
    CreateRoom {
        name: String
    },
    FetchRooms{}
}


pub fn fetch_rooms(swarm: &mut Swarm<ChatBehaviour>) {
    let key = kad::RecordKey::new(&"rooms".to_string());
    swarm.behaviour_mut().kademlia.get_record(key);
}


pub fn send_message(swarm: &mut Swarm<ChatBehaviour>, room: String, message: String) {
    let topic = gossipsub::IdentTopic::new(room);
    if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()) {
        log::info!("Error publishing: {:?}", err)
    }
}


pub fn request_file(swarm: &mut Swarm<ChatBehaviour>, message: String, peer: PeerId ) {
    swarm
        .behaviour_mut()
        .request_response
        .send_request(&peer, Request { message });
}


pub fn respond_file(swarm: &mut Swarm<ChatBehaviour>, filename: String, filepath: String, channel: ResponseChannel<Response>) {

    let data = std::fs::read(&filepath).unwrap_or_else(|_| Vec::new());
    
    swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, Response {filename, data })
        .expect("Connection to peer to be still open.");
}


pub fn update_rating(swarm: &mut Swarm<ChatBehaviour>, peer: PeerId, rating: i32, rating_update_queue: &mut HashMap<QueryId, (PeerId, i32)>) {
    let key_string = "rating_".to_string() + &peer.to_string();
    let key = kad::RecordKey::new(&key_string);
    let query_id = swarm.behaviour_mut().kademlia.get_record(key);
    rating_update_queue.insert(query_id, (peer, rating));
}

pub fn create_room(swarm: &mut Swarm<ChatBehaviour>, name: String) {

    let key = kad::RecordKey::new(&"rooms".to_string());
    let record = swarm.behaviour_mut().kademlia.store_mut().get(&key);
    
    if record.is_none() {

        let rooms = vec![name];
        let rooms_bytes = serde_cbor::to_vec(&rooms).unwrap();

        let record = kad::Record {
            key: kad::RecordKey::new(&key),
            value: rooms_bytes,
            publisher: None,
            expires: None,
        };

        swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("");

    } else {

        let mut rooms: Vec<String> = match serde_cbor::from_slice(&record.unwrap().value) {
            Ok(rooms) => rooms,
            Err(e) => {
                eprintln!("Failed to deserialize room list: {:?}", e);
                return;
            }
        };

        rooms.push(name.clone());

        let rooms_bytes = serde_cbor::to_vec(&rooms).unwrap();

        let record = kad::Record {
            key: kad::RecordKey::new(&key),
            value: rooms_bytes,
            publisher: None,
            expires: None,
        };

        swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("");
    }
}