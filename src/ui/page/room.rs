use crate::{ui::components::navbar, network::network::Client, state::STATE};
use tokio::io;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};

pub fn render(frame: &mut Frame) {

    let mut state = STATE.lock().unwrap();

    let room_key = state.current_room.clone();
    let room_chat_set = &mut state.messages;

    // Add chat to local storage if not already
    if !room_chat_set.contains_key(&room_key.clone()) {

        room_chat_set.entry(room_key).or_insert(vec![]);
        let room_key = &state.current_room.clone();

        // Add welcome message
        let msgs = state.messages.entry(room_key.clone()).or_default();
        msgs.push(format!("👋 Welcome to the {} room!", &room_key));
    }

    // Page layout
    let main_layout = navbar(frame);

    // Messages display
    let input_str: &str = &state.input;
    let room_key = state.current_room.clone();
    let msgs = &state.messages;
    let message_str: Vec<String> = msgs[&room_key].iter().map(|m| format!("{}", m)).collect();
    let concatenated_messages = message_str.join("\n");
    let messages = Paragraph::new(concatenated_messages)
        .block(
            Block::bordered()
                .title(format!("Chatting in room {} | <Esc> to Go Back", state.current_room))
                .style(Style::default().fg(Color::White))
        )
        .style(Style::default().fg(Color::White));

    // User input
    let input_display = Paragraph::new(input_str)
    .block(
        Block::bordered()
            .title("Type Message | <Enter> to send")
            .style(Style::default().fg(Color::Blue))
    )
    .style(Style::default().fg(Color::White));

    // Render
    frame.render_widget(messages, main_layout[1]);
    frame.render_widget(input_display, main_layout[2]);

}



pub async fn handle_events(client: &mut Client) -> io::Result<bool> {

    let mut state = STATE.lock().unwrap();

    let room_key = state.current_room.clone();
    client.join_room(room_key).await;

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Tab => {
                        state.tab = 2;
                    }
                    KeyCode::Esc => {
                        state.tab = 1;
                    }
                    KeyCode::Backspace => {
                        state.input.pop();
                    }
                    KeyCode::Char(c) => {
                        state.input.push(c)
                    }
                    KeyCode::Enter => {

                        {
                            let room_key = state.current_room.clone();
                            let message = state.input.to_string();
                            let nickname = state.nickname.clone();

                            let msgs = state.messages.entry(room_key.clone()).or_default();
                            msgs.push(format!("{}: {}", nickname, message.clone()));
                            client.send_message(message, room_key).await;
                        }
                        
                        state.input.clear();

                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}