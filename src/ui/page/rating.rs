use std::rc::Rc;

use crate::{network::client::Client, state::STATE, ui::components::Tab};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::*,
};

#[derive(Default)]
pub struct Rating {}

impl Rating{
    pub fn render(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {

        let state = STATE.lock().unwrap();
        let peer_id = state.current_rating.unwrap();
    
        let text = format!("\n\nYou have just received a file! \nGive {} a rating for this trade: \n\n1: Bad  2: Neutral  3: Good",
         state.nicknames.get(&peer_id.to_string()).unwrap());
         
        let notification = Paragraph::new(text)
            .block(
                Block::bordered()
                    .style(Style::default().fg(Color::Blue)).title("Rate a Peer")
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
    
        frame.render_widget(notification, layout[1]);
    }
    
    
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
    
            // Netrual rating
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