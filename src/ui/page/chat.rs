use std::rc::Rc;

use crate::{network::client::Client, state::STATE, ui::components::{input_component, message_component}};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
};

/// A page for users to chat with all other peers on the network.
#[derive(Default)]
pub struct Chat {
    input: String
}

impl Chat {

    /// Renders the page consisting of a display of sent and received messages, and an input field at the bottom of the
    /// page to allow the user to send messages.
    pub fn render(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {

        let (room_key, nickname) = self.room_setup();
        
        // Messages
        let messages = self.format_messages(room_key);
        let messages_display = message_component(&messages, nickname);
    
        // User input
        let input_display = input_component(self.input.as_str(), "Type Message | <Enter> to send".to_string());
    
        // Render
        frame.render_widget(messages_display, layout[1]);
        frame.render_widget(input_display, layout[2]);
    }
    
    
    /// Event handler for the Global Tab. Can type messages into the input at the bottom of the page, delete with 
    /// backspace and send with Enter.
    pub async fn handle_events(&mut self, client: &mut Client, key: KeyEvent) {
            
        match key.code {

            // User input into the message box
            KeyCode::Char(c) => {
                self.input.push(c)
            }

            // Allows for deletion of characters in the message box
            KeyCode::Backspace => {
                self.input.pop();
            }
    
            // Submit a message. Adds the message to the local message list and sends a
            // network request to share it with other peers subscribed to the topic.
            KeyCode::Enter => {
                
                let message = self.input.to_string();

                if !message.is_empty() {

                    let mut state = STATE.lock().unwrap();
                    let room_key = state.current_room.clone();
    
                    // Add to local storage
                    let msgs = state.messages.entry(room_key.clone()).or_default();
                    msgs.push(format!("{}: {}", "You".to_string(), message.clone()));

                    // Send message to the network
                    client.send_message(message, room_key).await;

                    self.input.clear();
                }
            }
            
            _ => {}
        }
    }


    /// Fetches messages for the room from the global store and formats them in a way to be displayed in the Ratatui UI.
    fn format_messages(&self, room: String) -> String {

        let state = STATE.lock().unwrap();
        let messages = state.messages.get(&room).expect("Rooms should be in the messages HashMap at all times");
        let message_str: Vec<String> = messages.iter().map(|m| format!("{}", m)).collect();
        let concatenated_messages = message_str.join("\n");

        concatenated_messages
    }


    /// Fetch relevant information about the room.
    fn room_setup(&self) -> (String, String) {
        
        let mut state = STATE.lock().unwrap();

        let nickname = state.nickname.clone();
        let room_key = state.current_room.clone();
        let room = state.current_room.clone();

        state.notifications.insert(room.clone(), false);

        if !state.messages.contains_key(&room_key) {
            state.messages.entry(room.clone()).or_insert(vec![format!("✨ Welcome to the {} chat!", &room)]);
        }
        
        (room_key, nickname)
    }

}