use std::{collections::HashMap, sync::{Arc, Mutex}};
use libp2p::PeerId;
use libp2p_request_response::ResponseChannel;
use ratatui::widgets::ListState;
use lazy_static::lazy_static;

use crate::network::Response;

pub struct GlobalState {
    pub nickname: String,
    pub has_added_nickname: bool,
    pub tab: usize,
    pub input: String,
    pub current_room: String,
    pub peers: Arc<Mutex<Vec<PeerId>>>,
    pub messages: Arc<Mutex<Vec<String>>>,
    pub rooms: Vec<String>,
    pub room_chats: HashMap<String, Vec<String>>,
    pub room_list_state: ListState,
    pub peer_list_state: ListState,
    pub request_list_state: ListState,
    pub nicknames: HashMap<String, String>,
    pub requests: Vec<(PeerId, String, ResponseChannel<Response>)>
}

impl GlobalState {

    fn new() -> Self {
        let mut state = Self {
            has_added_nickname: false,
            nickname: "Default".to_string(),
            tab: 5,
            input: String::new(),
            current_room: "global".to_string(),
            peers: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            room_chats: HashMap::new(),
            rooms: vec!["COSC473".to_string(), "COSC478".to_string(), "SENG406".to_string(), "SENG402".to_string()],
            room_list_state: ListState::default(), 
            peer_list_state: ListState::default(),
            request_list_state: ListState::default(),
            nicknames: HashMap::new(),
            requests: vec![],
        };

        // Initial Setup
        state.room_list_state.select_first();
        state.peer_list_state.select_first();
        state.request_list_state.select_first();
        state.messages.lock().unwrap().push("✨ Welcome to Global Chat!".to_string());
        state
    }
}


lazy_static! {
    pub static ref STATE: Arc<Mutex<GlobalState>> = Arc::new(Mutex::new(GlobalState::new()));
}