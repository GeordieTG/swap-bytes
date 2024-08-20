use libp2p::{multiaddr::Protocol, rendezvous, Multiaddr, Swarm};

use crate::network::network::ChatBehaviour;

pub async fn handle_event(event: libp2p::rendezvous::client::Event, swarm: &mut Swarm<ChatBehaviour>) {
    
    match event {

        rendezvous::client::Event::Discovered {
            registrations,
            cookie: _new_cookie,
            ..
        } => {
            // cookie.replace(new_cookie);
            for registration in registrations {
                for address in registration.record.addresses() {
                    let peer = registration.record.peer_id();
                    log::info!("Discovered peer {}: {}", peer, address);

                    let p2p_suffix = Protocol::P2p(peer);
                    let address_with_p2p =
                        if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                            address.clone().with(p2p_suffix)
                        } else {
                            address.clone()
                        };

                    swarm.dial(address_with_p2p).unwrap();
                }
            }
        }

        // once `/identify` did its job, we know our external address and can register
        rendezvous::client::Event::Registered {
            namespace,
            ttl,
            rendezvous_node,
        } => {
            log::info!(
                "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                namespace,
                rendezvous_node,
                ttl
            );
        }

        rendezvous::client::Event::RegisterFailed {
            rendezvous_node,
            namespace,
            error,
        } => {
            log::info!(
                "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                rendezvous_node,
                namespace,
                error
            );
            return;
        }
        
        _ => {}
    }
}