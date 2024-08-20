use libp2p_request_response::ResponseChannel;
use libp2p::{gossipsub, kad::QueryId, mdns, noise, request_response::{self, ProtocolSupport}, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, Multiaddr, PeerId, Swarm};
use futures::StreamExt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use libp2p::StreamProtocol;
use std::{error::Error, time::Duration};
use futures::channel::{mpsc, oneshot};
use futures::SinkExt;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use crate::network::mdns as mdns_events;
use crate::network::gossipsub as gossibsub_events;
use crate::network::kademlia as kademlia_events;
use crate::network::request_response as reqyest_response_events;


/// Defines the behaviour of our libp2p application
#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub request_response: request_response::cbor::Behaviour<Request, Response>,
    pub kademlia: kad::Behaviour<MemoryStore>
}


#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}


impl Client {
    
    /// Listen for incoming connections on the given address.
    pub(crate) async fn _start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }


    /// Send a given message the user has typed in the UI.
    pub(crate) async fn send_message(
        &mut self,
        message: String,
    ) {
        self.sender
            .send(Command::SendMessage { message })
            .await
            .expect("Command receiver not to be dropped.");
    }


    pub(crate) async fn send_request(
        &mut self,
        request: String,
        peer: PeerId
    ) {
        self.sender
            .send(Command::RequestFile { request, peer })
            .await
            .expect("Command receiver not to be dropped.");
    }

    pub(crate) async fn send_response(
        &mut self,
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    ) {
        self.sender
            .send(Command::RespondFile { filename, filepath, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }
}


#[derive(Debug)]
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    SendMessage {
        message: String,
    },
    RequestFile {
        request: String,
        peer: PeerId,
    },
    RespondFile {
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    },
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
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

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
    nickname_fetch_queue: HashMap<QueryId, (PeerId, String)>,
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
        }
    }

    /// Begins the libp2p event loop. To be called from the main application.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    None=>  return,
                },
            }
        }
    }


    /// Listens for incoming libp2p requests and handles them accordingly.
    async fn handle_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>) {

        match event {

            // Initial setup
            SwarmEvent::NewListenAddr { address, ..} => {
                log::info!("Listening on address: {address}");

                let peer_id = self.swarm.local_peer_id().clone();
                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, address);
            },

            // Handle MDNS events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(event)) => {
                mdns_events::handle_event(event, &mut self.swarm).await;
            }

            // Handle Gossipsub events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(event)) => {
                gossibsub_events::handle_event(event, &mut self.swarm, &mut self.nickname_fetch_queue).await;
            }

            // Handle Kademlia events
            SwarmEvent::Behaviour(ChatBehaviourEvent::Kademlia(event)) => {
                kademlia_events::handle_event(event, &mut self.nickname_fetch_queue).await;
            }
    
            // Handle Request-Response events
            SwarmEvent::Behaviour(ChatBehaviourEvent::RequestResponse(event)) => {
                reqyest_response_events::handle_event(event).await;
            }
        
            _ => {}
        }
    }


    /// These are commands that we can call from the UI to perform libp2p actions.
    /// Eg. when a user sends a message on the UI, we can communicate to our network running in the background through the mspc channel to instruct
    /// it to send the users message!
    async fn handle_command(&mut self, command: Command) {

        match command {

            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }

            Command::SendMessage { message } => {
                let topic = gossipsub::IdentTopic::new("global");
                if let Err(err) = self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()) {
                    log::info!("Error publishing: {:?}", err)
                }
            }

            Command::RequestFile {
                request,
                peer,
            } => {

                self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, Request { request });
            }

            Command::RespondFile { filename, filepath, channel } => {

                let data = std::fs::read(&filepath).unwrap_or_else(|_| Vec::new());

                log::info!("{:?}", data);
                
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, Response {filename, data })
                    .expect("Connection to peer to be still open.");
            }
        }
    }
}


/// Defines the properties sent when sharing a file with another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub request: String,
}


/// Defines the properties sent when acknowledging the reception of a shared file from another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub filename: String,
    pub data: Vec<u8>,
}