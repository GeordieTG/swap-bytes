use std::rc::Rc;

use crate::{network::client::Client, state::STATE, ui::components::{notification_component, Tab}};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
};

/// Page to rate a user after receiving a file from them. Will be displayed like a popup as soon as we receive a file.
#[derive(Default)]
pub struct Rating {}

impl Rating{

    /// Simply renders the page consisting of the notification that a user has sent you a file. The user is prompted to give them a rating
    /// depending on whether they recieved the correct file (Good, Neutral, Bad).
    pub fn render(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {

        let state = STATE.lock().unwrap();
        let peer_id = state.current_rating.unwrap();
    
        let text = format!("\n\nYou have just received a file! \nGive {} a rating for this trade: \n\n1: Bad  2: Neutral  3: Good",
         state.nicknames.get(&peer_id.to_string()).unwrap());
         
        let notification = notification_component(&text, "Rate a Peer".to_string());
    
        frame.render_widget(notification, layout[1]);
    }
    

    /// Event handler for the Rating page. Listens for user keystrokes.
    pub async fn handle_events<T: FnMut(Tab)>(&mut self, client: &mut Client, key: KeyEvent, mut switch_tab_callback: T) {
    
        let mut state = STATE.lock().unwrap();
        let peer_id = state.current_rating.unwrap();
    
        match key.code {
    
            // Bad rating
            KeyCode::Char('1') => {
                client.update_rating(peer_id, -1).await;
                state.current_rating = None;
                switch_tab_callback(Tab::Chat);
            }
    
            // Neutral rating
            KeyCode::Char('2') => {
                switch_tab_callback(Tab::Chat);
                state.current_rating = None;
            }
    
            // Good rating
            KeyCode::Char('3') => {
                client.update_rating(peer_id, 1).await;
                switch_tab_callback(Tab::Chat);
                state.current_rating = None;
            }
    
            _ => {}
        }
    }
}