use chrono::Local;
use fern::Dispatch;
use futures::StreamExt;
use swapbytes::network::network::{ChatBehaviour, ChatBehaviourEvent};
use std::{error::Error, fs::File};
use std::time::Duration;
use libp2p::{gossipsub, kad::QueryId, mdns, noise, ping, rendezvous, request_response::{self, ProtocolSupport}, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, Multiaddr, PeerId, Swarm};
use libp2p::{identify, StreamProtocol};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;


fn setup_logger() -> Result<(), Box<dyn Error>> {

    let log_file = File::create("src/log/rendezvous.log")?;
    Dispatch::new()
        .filter(|metadata| metadata.level() <= log::LevelFilter::Info)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .chain(log_file)
        .apply()?;

        log::info!("Application started");

    Ok(())
}


// As many functionality requires one node to be online to work, this node acts as a bootstrap node to ensure that the 
// network is always available
pub async fn create_node() -> Result<Swarm<ChatBehaviour>, Box<dyn Error>> {

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
                rendezvous: rendezvous::client::Behaviour::new(key.clone()),
                ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(7200)))
        .build();

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

    let topic = gossipsub::IdentTopic::new("global");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let external_address: Multiaddr = "/ip4/0.0.0.0/udp/62650/quic-v1".parse()?;
    swarm.listen_on(external_address.clone())?;

    Ok(swarm)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    setup_logger().expect("Logger setup failed");

    // Results in PeerID 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN which is
    // used as the rendezvous point by the other peer examples.
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32]).unwrap();

    let mut server = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| MyBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "rendezvous-example/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let _ = server.listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap());

    let mut peer = create_node().await.unwrap();
    log::info!("{}", peer.local_peer_id().to_string());

    run(&mut server, &mut peer).await;

    Ok(())
}


pub async fn run(server: &mut Swarm<MyBehaviour>, peer: &mut Swarm<ChatBehaviour>) {
    loop {
        tokio::select! {
            event = server.select_next_some() => handle_server_event(event).await,
            event = peer.select_next_some() => handle_peer_event(event).await,
        }
    }
}


// Handle events for the Rendezvous server
pub async fn handle_server_event(event: SwarmEvent<MyBehaviourEvent>) {

    match event {
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            log::info!("Connected to {}", peer_id);
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            log::info!("Disconnected from {}", peer_id);
        }
        SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
            rendezvous::server::Event::PeerRegistered { peer, registration },
        )) => {
            log::info!(
                "Peer {} registered for namespace '{}'",
                peer,
                registration.namespace
            );
        }
        SwarmEvent::Behaviour(MyBehaviourEvent::Rendezvous(
            rendezvous::server::Event::DiscoverServed {
                enquirer,
                registrations,
            },
        )) => {
            log::info!(
                "Served peer {} with {} registrations",
                enquirer,
                registrations.len()
            );
        }
        other => {
            log::debug!("Unhandled {:?}", other);
        }
    }
}


// Handle events for the bootstrap node
pub async fn handle_peer_event(event: SwarmEvent<ChatBehaviourEvent>) {
    
    match event {

        SwarmEvent::NewListenAddr { address, ..} => {
            log::info!("Bootstrap node listening on address: {address}");
        },

        _ => {}
    }
}

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}