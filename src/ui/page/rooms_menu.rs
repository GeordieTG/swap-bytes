use std::rc::Rc;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::*,
};

use crate::{network::client::Client, state::STATE, ui::components::{input_component, list_component, Tab}};

/// A page for users to view all available rooms on the network and select one to join.
pub struct RoomMenu {
    input: String,
    room_list_state: ListState,
}


/// A default implementation of the RoomMenu. Allows for the first item in the list to be selected.
impl Default for RoomMenu {
    fn default() -> Self {
        let mut menu = Self {
            input: String::new(),
            room_list_state: ListState::default(),
        };
        menu.room_list_state.select_first();
        menu
    }
}

impl RoomMenu {

    /// Simply renders the page consisting of a list of availabe rooms, and an input field at the bottom of the page to allow the
    /// user to create new rooms on the network.
    pub fn render(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {

        // Room list display
        let room_items = self.format_rooms();
        let rooms_display = list_component(room_items, "📚 Select Room to Enter".to_string());
     
        // Create room option
        let input_display = input_component(self.input.as_str(), "Type new room name | Create new room <Right Arrow>".to_string());
    
        // Render
        frame.render_stateful_widget(rooms_display, layout[1], &mut self.room_list_state.clone());
        frame.render_widget(input_display, layout[2]);
    }
    
    
    /// Event handler for the RoomMenu Tab. Can use Up and Down Arrows to navigate the list of rooms and Enter to select a room to join.
    /// The user also has the ability to create a new room by typing into the input field and push Right Arrow to confirm the creation.
    pub async fn handle_events<T: FnMut(Tab)>(&mut self, client: &mut Client, key: KeyEvent, mut switch_tab_callback: T) {
       
        match key.code {
            
            // Navigate up the room list
            KeyCode::Up => {
                self.room_list_state.select_previous();
            }
            
            // Navigate down the room list
            KeyCode::Down => {
                let state = STATE.lock().unwrap();
                if self.room_list_state.selected().unwrap() != state.rooms.len() - 1 {
                    self.room_list_state.select_next();
                }
            }

            // Allows for deletion of characters in the room creation box
            KeyCode::Backspace => {
                self.input.pop();
            }

            // User input into the room creation box
            KeyCode::Char(c) => {
                self.input.push(c)
            }

            // Select room
            KeyCode::Enter => {
                let mut state = STATE.lock().unwrap();
                state.current_room = state.rooms.get(self.room_list_state.selected().expect("")).expect("").to_string();
                switch_tab_callback(Tab::Chat);
            }

            // Create room based on current input
            KeyCode::Right => {
                let state = STATE.lock().unwrap();
                if self.input != String::new() && !state.rooms.contains(&self.input) {
                    client.create_room(self.input.to_string()).await;
                    self.input = String::new();
                }
            }

            _ => {}
        }
    }

    /// Fetches available rooms from the global store and formats them in a way to be displayed in the Ratatui UI.
    /// Will display with "- New Messages" if the room has unread messages.
    fn format_rooms(&self) -> Vec<ListItem> {

        let state = STATE.lock().unwrap();

        let room_items: Vec<ListItem> = state.rooms.iter().map(|room| {
        
            let notification = state.notifications.get(room);

            if notification == Some(&true) {
                ListItem::new(format!("{} - New Messages", room.as_str()))
            } else {
                ListItem::new(format!("{}", room.as_str()))
            }
        
        }).collect();

        room_items
    } 
}