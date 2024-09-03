use libp2p::{gossipsub, mdns, noise, request_response::{self, ProtocolSupport}, swarm::NetworkBehaviour, tcp, yamux, Multiaddr};
use serde::{Serialize, Deserialize};
use libp2p::StreamProtocol;
use std::{error::Error, time::Duration};
use futures::channel::mpsc;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;

use crate::state::STATE;

use super::{client::Client, event_loop::EventLoop};

/// Main network entry point. Defines the behaviour of our libp2p application
#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub request_response: request_response::cbor::Behaviour<Request, Response>,
    pub kademlia: kad::Behaviour<MemoryStore>
}


/// Defines the properties sent when requesting a file from another user.
/// Simply just a message (eg. Hey Ben, can I have last weeks COSC473 Notes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub message: String,
}


/// Defines the properties sent when sharing a file with another user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub filename: String,
    pub data: Vec<u8>,
}


/// Sets up a new libp2p swarm and returns an EventLoop and Client to be used in the main program
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

        let mut state = STATE.lock().unwrap();
        state.peer_id = swarm.local_peer_id().to_string();

        Ok((
            Client {
                sender: command_sender,
            },
            EventLoop::new(swarm, command_receiver),
        ))
}