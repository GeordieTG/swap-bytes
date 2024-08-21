use std::collections::HashMap;
use libp2p::{kad::{self, QueryId}, PeerId, Swarm};
use serde::Deserialize;
use crate::{network::network::ChatBehaviour, state::STATE};

#[derive(Deserialize)]
#[serde(untagged)] // This allows deserialization of either variant.
enum Value {
    Nickname(String),
    Rating(i32),
}


pub async fn handle_event(
    event: libp2p::kad::Event,
    nickname_fetch_queue: &mut HashMap<QueryId, PeerId>,
    rating_fetch_queue: &mut HashMap<QueryId, (String, String, String)>,
    rating_update_queue: &mut HashMap<QueryId, (PeerId, i32)>,
    swarm: &mut Swarm<ChatBehaviour>
    ) {

    match event {

        kad::Event::OutboundQueryProgressed { result, id, ..} => {
                
            match result {

                kad::QueryResult::GetRecord(Ok(
                    kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                        record: kad::Record { key, value, ..},
                        ..
                    })
                )) => {
                    match serde_cbor::from_slice::<Value>(&value) {
                        
                        Ok(Value::Nickname(nickname)) => {
                            log::info!("Got record {:?} {:?}", std::str::from_utf8(key.as_ref()).unwrap(), value);

                            if nickname_fetch_queue.contains_key(&id) {
                                
                                let mut state = STATE.lock().unwrap();
                                let peer_id = nickname_fetch_queue.remove(&id).expect("Message was not in queue");
                                state.nicknames.insert(peer_id.to_string(), nickname);

                            }                           
                        }

                        Ok(Value::Rating(rating)) => {

                            if rating_fetch_queue.contains_key(&id) {

                                let (message, nickname, topic) = rating_fetch_queue.remove(&id).expect("Message was not in queue");
                                
                                let msg = if rating > 0 {
                                    format!("{} {}: {}", "😇", nickname, message)
                                } else if rating < 0 {
                                    format!("{} {}: {}", "👿", nickname, message)
                                } else {
                                    format!("{}: {}", nickname, message)
                                };

                                let mut state = STATE.lock().unwrap();

                                if topic == "global".to_string() {
                                    state.messages.lock().unwrap().push(msg);
                                } else {
                                    state.room_chats.get_mut(&topic.to_string()).expect("").push(msg);
                                }
                            } else if rating_update_queue.contains_key(&id) {
                                
                                let (peer_id, adjustment) = rating_update_queue.remove(&id).expect("Message was not in queue");

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

                        Err(e) => {
                            log::info!("Error deserializing {e:?}");
                        }
                    }
                }

                kad::QueryResult::GetRecord(Err(err)) => {
                    log::info!("Failed to get record {err:?}");
                }

                kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                    log::info!("Successfully put record {:?}", std::str::from_utf8(key.as_ref()).unwrap());
                }

                kad::QueryResult::PutRecord(Err(err)) => {
                    log::info!("Failed to put record {err:?}");
                }

                _ => {}

            }
        }

        _ => {}
    }
}