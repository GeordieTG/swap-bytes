use libp2p::{gossipsub, kad::QueryId, swarm::SwarmEvent, Multiaddr, PeerId, Swarm};
use futures::StreamExt;
use std::collections::HashMap;
use futures::channel::mpsc;
use libp2p::kad;
use crate::{network::behaviour::mdns as mdns_events, state::STATE};
use crate::network::behaviour::gossipsub as gossibsub_events;
use crate::network::behaviour::kademlia as kademlia_events;
use crate::network::behaviour::request_response as reqyest_response_events;

use super::{client::Client, command::*, network::{ChatBehaviour, ChatBehaviourEvent}};


/// Defines the libp2p event loop. 
pub struct EventLoop {
    swarm: Swarm<ChatBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    nickname_fetch_queue: HashMap<QueryId, PeerId>,
    rating_fetch_queue: HashMap<QueryId, (String, String, String)>, // (PeerId, Message, Nickname, Topic)
    rating_update_queue: HashMap<QueryId, (PeerId, i32)> // (PeerId, Recent Rating)
}


/// Encapsulates the libp2p event listening and will perform corresponding functionality when certain events occur.
/// For example, it will listen for incoming messages, and then append the received message to the global store to be shown in the UI.
/// This is intended to be run in the background of the application asyncronously.
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
                self.setup(address);
            },

            // Handle MDNS (Peer Connection) events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(event)) => {
                mdns_events::handle_event(event, &mut self.swarm, &mut self.nickname_fetch_queue).await;
            }

            // Handle Gossipsub (Message) events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(event)) => {
                gossibsub_events::handle_event(event, &mut self.rating_fetch_queue, &mut self.swarm).await;
            }

            // Handle Kademlia (Stored DHT) events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Kademlia(event)) => {
                kademlia_events::handle_event(event, &mut self.nickname_fetch_queue, &mut self.rating_fetch_queue, &mut self.rating_update_queue, &mut self.swarm).await;
            }
    
            // Handle Request-Response (File-Sharing) events
            SwarmEvent::Behaviour(ChatBehaviourEvent::RequestResponse(event)) => {
                reqyest_response_events::handle_event(event).await;
            }

            other => {
                log::info!("{:?}", other);
            }
        }
    }


    /// These are commands that we can call from the UI to perform libp2p actions.
    /// Eg. when a user sends a message on the UI, we can communicate to our network running in the background through the mspc channel to instruct
    /// it to send the users message!
    async fn handle_command(&mut self, command: Command) {

        match command {

            Command::FetchRooms {  } => {
                fetch_rooms(&mut self.swarm);
            }

            Command::CreateRoom { name } => {
                create_room(&mut self.swarm, name)
            }

            Command::SendMessage { message , room} => {
                send_message(&mut self.swarm, room, message)
            }

            Command::RequestFile {message, peer} => {
                request_file(&mut self.swarm, message, peer);
            }

            Command::RespondFile { filename, filepath, channel } => {
                respond_file(&mut self.swarm, filename, filepath, channel);
            }

            Command::UpdateRating { peer, rating } => {
                update_rating(&mut self.swarm, peer, rating, &mut self.rating_update_queue)
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
}