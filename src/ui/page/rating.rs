use crate::{network::network::Client, state::STATE};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};


pub fn render(frame: &mut Frame) {

    // Page layout
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ],
    )
    .split(frame.area());

    let state = STATE.lock().unwrap();
    let peer_id = state.current_rating.unwrap();
    let text = format!("\n\nYou have just received a file! \nGive {} a rating for this trade: \n\n1: Bad  2: Neutral  3: Good", state.nicknames.get(&peer_id.to_string()).unwrap());
    let notification = Paragraph::new(text)
        .block(
            Block::bordered()
                .style(Style::default().fg(Color::Blue)).title("Rate a Peer")
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    // Render
    frame.render_widget(notification, main_layout[1]);
}



pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {

    let mut state = STATE.lock().unwrap();
    let peer_id = state.current_rating.unwrap();

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Char('1') => {
                        state.tab = 2;
                        client.update_rating(peer_id, -1).await;
                    }
                    KeyCode::Char('2') => {
                        state.tab = 2;
                    }
                    KeyCode::Char('3') => {
                        state.tab = 2;
                        client.update_rating(peer_id, 1).await;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}