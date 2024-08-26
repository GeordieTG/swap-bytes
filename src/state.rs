use std::{collections::HashMap, sync::{Arc, Mutex}};
use libp2p::PeerId;
use libp2p_request_response::ResponseChannel;
use ratatui::widgets::ListState;
use lazy_static::lazy_static;

use crate::network::network::Response;

// This is the application state for an indivdual peer
pub struct GlobalState {
    pub nickname: String,
    pub tab: usize,
    pub input: String,
    pub current_room: String,
    pub peers: Arc<Mutex<Vec<PeerId>>>,
    pub rooms: Vec<String>,
    pub messages: HashMap<String, Vec<String>>,
    pub room_list_state: ListState,
    pub peer_list_state: ListState,
    pub request_list_state: ListState,
    pub nicknames: HashMap<String, String>,
    pub requests: Vec<(PeerId, String, ResponseChannel<Response>)>,
    pub current_rating: Option<PeerId>,
}

// This sets up the state with default values
impl GlobalState {

    fn new() -> Self {
        let mut state = Self {
            nickname: String::new(),
            tab: 5,
            input: String::new(),
            peers: Arc::new(Mutex::new(Vec::new())),
            messages: HashMap::new(),
            rooms: vec!["COSC473".to_string(), "COSC478".to_string(), "SENG406".to_string(), "SENG402".to_string()],
            room_list_state: ListState::default(), 
            peer_list_state: ListState::default(),
            request_list_state: ListState::default(),
            nicknames: HashMap::new(),
            requests: vec![],
            current_rating: None,
            current_room: "global".to_string(),
        };

        state.room_list_state.select_first();
        state.peer_list_state.select_first();
        state.request_list_state.select_first();
        state.messages.insert("global".to_string(), vec!["✨ Welcome to Global Chat!".to_string()]);
        state
    }
}


// This creates a static instance that can be accessed and updated globally
lazy_static! {
    pub static ref STATE: Arc<Mutex<GlobalState>> = Arc::new(Mutex::new(GlobalState::new()));
}