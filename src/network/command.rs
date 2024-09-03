use std::collections::HashMap;

use libp2p::{gossipsub, kad::{self, store::RecordStore, QueryId}, PeerId, Swarm};
use libp2p_request_response::ResponseChannel;

use super::network::{ChatBehaviour, Request, Response};


/// These are commands that can be called from the UI to instruct the libp2p network to perform an action.
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


/// Fetch all currently available rooms to join. Result will come as an OutboundQueryProgressedEvent (see kademlia.rs).
pub fn fetch_rooms(swarm: &mut Swarm<ChatBehaviour>) {
    let key = kad::RecordKey::new(&"rooms".to_string());
    swarm.behaviour_mut().kademlia.get_record(key);
}


/// Publish a message to a given topic.
pub fn send_message(swarm: &mut Swarm<ChatBehaviour>, room: String, message: String) {
    let topic = gossipsub::IdentTopic::new(room);
    if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()) {
        log::info!("Error publishing: {:?}", err)
    }
}


/// Request a file from another user with a message (eg. Can I have last weeks COSC473 notes).
pub fn request_file(swarm: &mut Swarm<ChatBehaviour>, message: String, peer: PeerId ) {
    swarm
        .behaviour_mut()
        .request_response
        .send_request(&peer, Request { message });
}


/// Send a file at the given the filepath to the user who requested it.
pub fn respond_file(swarm: &mut Swarm<ChatBehaviour>, filename: String, filepath: String, channel: ResponseChannel<Response>) {

    let data = std::fs::read(&filepath).unwrap_or_else(|_| Vec::new());
    
    swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, Response {filename, data })
        .expect("Connection to peer to be still open.");
}


/// Update the rating of a peer. Will add it to a queue as the rating first needs to be fetched from the DHT before modifying it. 
/// The fetch result will come as an OutboundQueryProgressedEvent and the rest of the update will happen after (see kademlia.rs).
pub fn update_rating(swarm: &mut Swarm<ChatBehaviour>, peer: PeerId, rating: i32, rating_update_queue: &mut HashMap<QueryId, (PeerId, i32)>) {
    let key_string = "rating_".to_string() + &peer.to_string();
    let key = kad::RecordKey::new(&key_string);
    let query_id = swarm.behaviour_mut().kademlia.get_record(key);
    rating_update_queue.insert(query_id, (peer, rating));
}


/// Create a new room to be shared across the network.
pub fn create_room(swarm: &mut Swarm<ChatBehaviour>, name: String) {

    let key = kad::RecordKey::new(&"rooms".to_string());
    let record = swarm.behaviour_mut().kademlia.store_mut().get(&key);
    
    // If this is the first "created" room, there won't be a "rooms" record in the DHT yet.
    let rooms = if record.is_none() {
        vec![name]
    } else {
        let mut rooms: Vec<String> = serde_cbor::from_slice(&record.unwrap().value).unwrap();
        rooms.push(name.clone());
        rooms
    };

    let rooms_bytes = serde_cbor::to_vec(&rooms).unwrap();

    let record = kad::Record {
        key: kad::RecordKey::new(&key),
        value: rooms_bytes,
        publisher: None,
        expires: None,
    };

    swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("");
    
}