use std::collections::HashMap;
use libp2p::{kad::{self, QueryId}, PeerId};
use crate::state::STATE;


pub async fn handle_event(event: libp2p::kad::Event, nickname_fetch_queue: &mut HashMap<QueryId, (PeerId, String, String)>) {

    match event {

        kad::Event::OutboundQueryProgressed { result, id, ..} => {
                
            match result {

                kad::QueryResult::GetRecord(Ok(
                    kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                        record: kad::Record { key, value, ..},
                        ..
                    })
                )) => {
                    match serde_cbor::from_slice::<String>(&value) {
                        
                        Ok(nickname) => {
                            log::info!("Got record {:?} {:?}", std::str::from_utf8(key.as_ref()).unwrap(), nickname);

                            let mut state = STATE.lock().unwrap();

                            if nickname_fetch_queue.contains_key(&id) {
                                let (peer_id, message, topic) = nickname_fetch_queue.remove(&id).expect("Message was not in queue");

                                if topic == "global".to_string() {
                                    state.messages.lock().unwrap().push(format!("{}: {}", nickname, message));
                                } else {
                                    state.room_chats.get_mut(&topic.to_string()).expect("").push(format!("{}: {}", nickname, message));
                                }
                                state.nicknames.insert(peer_id.to_string(), nickname);
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