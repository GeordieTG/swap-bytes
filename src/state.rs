use std::{collections::HashMap, sync::{Arc, Mutex}};
use libp2p::PeerId;
use libp2p_request_response::ResponseChannel;
use lazy_static::lazy_static;

use crate::network::network::Response;


/// The state of our application. Includes items such as the users nickname, a list of all connected pairs,
/// a store of all messages for each room and so on.
#[derive(Default)]
pub struct GlobalState {
    pub nickname: String,
    pub nicknames: HashMap<String, String>,
    pub peers: Vec<PeerId>,
    pub rooms: Vec<String>,
    pub messages: HashMap<String, Vec<String>>,
    pub requests: Vec<(PeerId, String, ResponseChannel<Response>)>,
    pub notifications: HashMap<String, bool>,
    pub current_rating: Option<PeerId>,
    pub current_room: String,
}

impl GlobalState {

    /// Sets the initial values of the Global State (Specifically the current room and the default rooms).
    fn new() -> GlobalState {
    
        let mut state = GlobalState::default();
    
        let default_rooms = &mut vec!["Global".to_string(), "COSC473".to_string(), "COSC478".to_string(), "SENG406".to_string(), "SENG402".to_string()];
        let room_key = "Global".to_string();
    
        state.current_room = room_key.clone();
        state.rooms.append(default_rooms);
    
        state
    }
}


// Creates a static instance of the GlobalState to be accessed throughout the application.
lazy_static! {
    pub static ref STATE: Arc<Mutex<GlobalState>> = Arc::new(Mutex::new(GlobalState::new()));
}