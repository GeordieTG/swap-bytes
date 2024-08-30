use libp2p_request_response::ResponseChannel;
use libp2p::{gossipsub, kad::{store::RecordStore, QueryId}, mdns, noise, request_response::{self, ProtocolSupport}, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, Multiaddr, PeerId, Swarm};
use futures::StreamExt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use libp2p::StreamProtocol;
use std::{error::Error, time::Duration};
use futures::channel::{mpsc, oneshot};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use crate::{network::behaviour::mdns as mdns_events, state::STATE};
use crate::network::behaviour::gossipsub as gossibsub_events;
use crate::network::behaviour::kademlia as kademlia_events;
use crate::network::behaviour::request_response as reqyest_response_events;

use super::client::Client;

/// Defines the behaviour of our libp2p application
#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub request_response: request_response::cbor::Behaviour<Request, Response>,
    pub kademlia: kad::Behaviour<MemoryStore>
}


/// Defines the properties sent when sharing a file with another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub message: String,
}


/// Defines the properties sent when acknowledging the reception of a shared file from another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub filename: String,
    pub data: Vec<u8>,
}


#[derive(Debug)]
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
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


/// Sets up a new libp2p swarm and returns an EventLoop to be used in the main program
pub fn new() -> Result<(Client, EventLoop), Box<dyn Error>> {

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new, 
            yamux::Config::default
        )?
        .with_quic()
        .with_behaviour(|key | {
            Ok(ChatBehaviour {
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(), 
                    key.public().to_peer_id()
                )?,
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::Config::default(),
                )?,
                request_response: request_response::cbor::Behaviour::new(
                    [(
                        StreamProtocol::new("/file-exchange/1"),
                        ProtocolSupport::Full,
                    )],
                    request_response::Config::default().with_request_timeout(Duration::from_secs(7200)),
                ),
                kademlia: kad::Behaviour::new(key.public().to_peer_id(), MemoryStore::new(key.public().to_peer_id())),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(7200)))
        .build();

        swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

        let topic = gossipsub::IdentTopic::new("global");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let external_address: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;
        swarm.listen_on(external_address.clone())?;

        let (command_sender, command_receiver) = mpsc::channel(0);

        Ok((
            Client {
                sender: command_sender,
            },
            EventLoop::new(swarm, command_receiver),
        ))
}


/// Defines the libp2p event loop. 
/// Consists of the Swarm to perform network tasks, as well as the global state of messages and connected peers to update when need be.
pub struct EventLoop {
    swarm: Swarm<ChatBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    nickname_fetch_queue: HashMap<QueryId, PeerId>,
    rating_fetch_queue: HashMap<QueryId, (String, String, String)>, // (PeerId, Message, Nickname, Topic)
    rating_update_queue: HashMap<QueryId, (PeerId, i32)> // (PeerId, Recent Rating)
}


/// Encapsulates the libp2p event listening and will perform corresponding functionality when certain events occur.
/// For example, it will listen for incoming messages, and then append the received message to the global store to be shown in the UI.
/// This is intended to run in the background of the application asyncronously.
impl EventLoop {
    pub fn new(
        swarm: Swarm<ChatBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            nickname_fetch_queue: HashMap::new(),
            rating_fetch_queue: HashMap::new(),
            rating_update_queue: HashMap::new(),
        }
    }

    /// Begins the libp2p event loop. To be called from the main application.
    pub async fn run(mut self, client: Client) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event, &mut client.clone()).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    None=>  return,
                },
            }
        }
    }


    /// Listens for incoming libp2p requests and handles them accordingly.
    async fn handle_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>, mut _client: &mut Client) {

        match event {

            // Initial setup
            SwarmEvent::NewListenAddr { address, ..} => {
                log::info!("Listening on address: {address}");
                self.setup(address);
            },

            // Handle MDNS events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(event)) => {
                mdns_events::handle_event(event, &mut self.swarm, &mut self.nickname_fetch_queue).await;
            }

            // Handle Gossipsub events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(event)) => {
                gossibsub_events::handle_event(event, &mut self.rating_fetch_queue, &mut self.swarm).await;
            }

            // Handle Kademlia events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Kademlia(event)) => {
                kademlia_events::handle_event(event, &mut self.nickname_fetch_queue, &mut self.rating_fetch_queue, &mut self.rating_update_queue, &mut self.swarm).await;
            }
    
            // Handle Request-Response events
            SwarmEvent::Behaviour(ChatBehaviourEvent::RequestResponse(event)) => {
                reqyest_response_events::handle_event(event).await;
            }

            other => {
                log::info!("Unhandled {:?}", other);
            }
        }
    }


    // Sets up a user when they first join the network.
    // Adds their nickname and assigns an inital peer rating of 0 to the Kademlia DHT.
    fn setup(&mut self, address: Multiaddr) {

        if address.to_string().contains("/ip4/127.0.0.1/udp") {

            let peer_id = self.swarm.local_peer_id().clone();
            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, address);

            // Add your nickname to DHT
            let mut state= STATE.lock().unwrap();
            let nickname_bytes = serde_cbor::to_vec(&state.nickname).unwrap();
            let key = "nickname_".to_string() + &peer_id.to_string();

            let record = kad::Record {
                key: kad::RecordKey::new(&key),
                value: nickname_bytes,
                publisher: None,
                expires: None,
            };

            self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record");

            // Add your peer rating to DHT
            let key = "rating_".to_string() + &peer_id.to_string();
            let rating_bytes = serde_cbor::to_vec(&0).unwrap();

            let record = kad::Record {
                key: kad::RecordKey::new(&key),
                value: rating_bytes,
                publisher: None,
                expires: None,
            };

            self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record");


            // Connect to the default rooms
            let default_rooms = &mut vec!["Global".to_string(), "COSC473".to_string(), "COSC478".to_string(), "SENG406".to_string(), "SENG402".to_string()];
            for room in default_rooms {
                let topic = gossipsub::IdentTopic::new(room.to_string());
                self.swarm.behaviour_mut().gossipsub.subscribe(&topic).expect("");

                state.messages.entry(room.clone()).or_insert(vec![]);
    
                let msgs = state.messages.entry(room.clone()).or_default();
                msgs.push(format!("✨ Welcome to the {} chat!", &room));
            }   
        }
    }


    /// These are commands that we can call from the UI to perform libp2p actions.
    /// Eg. when a user sends a message on the UI, we can communicate to our network running in the background through the mspc channel to instruct
    /// it to send the users message!
    async fn handle_command(&mut self, command: Command) {

        match command {

            Command::FetchRooms {  } => {
                let key = kad::RecordKey::new(&"rooms".to_string());
                self.swarm.behaviour_mut().kademlia.get_record(key);
            }

            Command::CreateRoom { name } => {
                let key = kad::RecordKey::new(&"rooms".to_string());
                let record = self.swarm.behaviour_mut().kademlia.store_mut().get(&key);
                
                if record.is_none() {

                    let rooms = vec![name];
                    let rooms_bytes = serde_cbor::to_vec(&rooms).unwrap();
    
                    let record = kad::Record {
                        key: kad::RecordKey::new(&key),
                        value: rooms_bytes,
                        publisher: None,
                        expires: None,
                    };

                    self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("");

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
    
                    self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("");
                }
            }

            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }

            Command::SendMessage { message , room} => {
                let topic = gossipsub::IdentTopic::new(room);
                if let Err(err) = self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()) {
                    log::info!("Error publishing: {:?}", err)
                }
            }

            Command::RequestFile {
                message,
                peer,
            } => {

                self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, Request { message });
            }

            Command::RespondFile { filename, filepath, channel } => {

                let data = std::fs::read(&filepath).unwrap_or_else(|_| Vec::new());
                
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, Response {filename, data })
                    .expect("Connection to peer to be still open.");
            }

            Command::UpdateRating { peer, rating } => {
                // Fetch the users nickname from the DHT
                let key_string = "rating_".to_string() + &peer.to_string();
                let key = kad::RecordKey::new(&key_string);
                let query_id = self.swarm.behaviour_mut().kademlia.get_record(key);
                self.rating_update_queue.insert(query_id, (peer, rating));
            }
        }
    }
}