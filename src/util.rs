// Given two PeerId's participating in a DM, formats the room key for the chat to uniquely identify it and ensure consistancy.
pub fn format_dm_key(peer_id: String, own_peer_id: String) -> String {

    let (bigger_key, smaller_key) = if peer_id > own_peer_id {
        (peer_id, own_peer_id)
    } else {
        (own_peer_id, peer_id)
    };

    format!("{}_{}", bigger_key, smaller_key)
}