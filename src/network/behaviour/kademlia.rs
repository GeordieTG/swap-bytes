use std::collections::HashMap;
use libp2p::{gossipsub, kad::{self, QueryId}, PeerId, Swarm};
use serde::Deserialize;
use crate::{network::network::ChatBehaviour, state::STATE};

/// Defines the different types of values stored in the Kademlia DHT.
#[derive(Deserialize)]
#[serde(untagged)]
enum Value {
    Nickname(String),
    Rating(i32),
    Rooms(Vec<String>)
}


/// Handles all Kademlia events that come through the network event loop.
pub async fn handle_event(
    event: libp2p::kad::Event,
    nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String)>,
    rating_fetch_queue: &mut HashMap<QueryId, (String, String, String)>,
    rating_update_queue: &mut HashMap<QueryId, (PeerId, i32)>,
    swarm: &mut Swarm<ChatBehaviour>
    ) {

    match event {

        // Whenever a Kademlia "get_record" is called, the result will come back as an OutboundQueryProgressed Event. 
        kad::Event::OutboundQueryProgressed { result, id, ..} => {
                
            match result {

                kad::QueryResult::GetRecord(Ok(
                    kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                        record: kad::Record { key, value, ..},
                        ..
                    })
                )) => {

                    match serde_cbor::from_slice::<Value>(&value) {
                        
                        // If the returned value was of type Nickname, this means another users nickname has been fetched from the DHT (this will have
                        // been called on connection with another peer). After the fetch, we store the nickname for the peer in our local storage and
                        // add a personalised message to their direct message. 
                        Ok(Value::Nickname(nickname)) => {
                            log::info!("Got record {:?} {:?}", std::str::from_utf8(key.as_ref()).unwrap(), value);

                            if nickname_fetch_queue.contains_key(&id) {

                                // Add nickname to local storage
                                let (peer_id, dm_key) = nickname_fetch_queue.remove(&id).unwrap();
                                let mut state = STATE.lock().unwrap();
                                state.nicknames.insert(peer_id.to_string(), nickname);

                                // Add personalised message to the user's direct message.
                                let nickname = state.nicknames.get(&peer_id.to_string()).unwrap().clone();      
                                state.messages.insert(dm_key.clone(), vec![format!("😀 Chatting with {}", nickname.clone())]);
                            }                           

                        }

                        // If the returned value was of type Rating, this means another users rating has been fetched from the DHT. This is used in two senarios,
                        // 1 - When we receive a message from another user and want to display their most up to date rating, and 2 - when we want to update the peers
                        // rating after recieving a file from them.
                        Ok(Value::Rating(rating)) => {

                            // In the event we have recieved a message and simply want to fetch the users rating, the message will be in the rating_fetch_queue (See gossibsub.rs).
                            // The queue contains the message information with a Kademlia QueryID which is matched to the QueryID of this rating fetch. The newly created message (with the rating) 
                            // is appended to the messages list for the room to be displayed.
                            if rating_fetch_queue.contains_key(&id) {

                                let (message, nickname, topic) = rating_fetch_queue.remove(&id).unwrap();
                                
                                let msg = if rating > 0 {
                                    format!("{} {}: {}", "😇", nickname, message)
                                } else if rating < 0 {
                                    format!("{} {}: {}", "👿", nickname, message)
                                } else {
                                    format!("{}: {}", nickname, message)
                                };

                                let mut state = STATE.lock().unwrap();
                                state.messages.get_mut(&topic.to_string()).expect("").push(msg);
                                
                            // In the event we have just given a rating to another peer after a trade, the peer_id and the rating will be in the rating_update_queue (see rating.rs).
                            // The queue contains the rating information with a Kademlia QueryID which is matched to the QueryID of this rating fetch. The fetched rating is updated (either +1 or -1)
                            // and pushed back to the DHT.
                            } else if rating_update_queue.contains_key(&id) {
                                
                                let (peer_id, adjustment) = rating_update_queue.remove(&id).unwrap();

                                let key = "rating_".to_string() + &peer_id.to_string();
                                let new_rating = rating + adjustment;
                                let rating_bytes = serde_cbor::to_vec(&new_rating).unwrap();

                                let record = kad::Record {
                                    key: kad::RecordKey::new(&key),
                                    value: rating_bytes,
                                    publisher: None,
                                    expires: None,
                                };

                                swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record");

                            }
                        }

                        // If the returned value was of type Rooms, this means that the up to date list of rooms has been fetched from the DHT (this will have
                        // been called on the entry of the "Rooms" tab). After the fetch, we store the updated list of rooms in our local storage. It is important
                        // to note that only "created" rooms are stored in the DHT, not the default rooms.
                        Ok(Value::Rooms(mut rooms)) => {

                            // Updates the rooms list with created AND default rooms
                            let mut state = STATE.lock().unwrap();
                            let mut default_rooms = vec!["Global".to_string(), "COSC473".to_string(), "COSC478".to_string(), "SENG406".to_string(), "SENG402".to_string()];
                            rooms.append(&mut default_rooms);
                            state.rooms = rooms.clone();


                            // Automatically subscribes the user to all rooms
                            for room in rooms {
                    
                                let topic = gossipsub::IdentTopic::new(room.to_string());
                                swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

                                if !state.messages.contains_key(&room) {
                                    state.messages.entry(room.clone()).or_insert(vec![format!("✨ Welcome to the {} chat!", &room)]);
                                }
                            }   
                        }

                        Err(e) => {
                            log::info!("Error deserializing {e:?}");
                        }
                    }
                }

                other => {
                    log::info!("{:?}", other);
                }
            }
        }
        other => {
            log::info!("{:?}", other);
        }
    }
}