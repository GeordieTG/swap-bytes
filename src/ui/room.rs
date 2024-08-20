use crate::{network::network::Client, state::STATE};
use tokio::io;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};

pub fn render(frame: &mut Frame) {

    let mut state = STATE.lock().unwrap();

    let room_key = state.current_room.clone();
    let room_chat_set = &mut state.room_chats;

    // Add chat to local storage if not already
    if !room_chat_set.contains_key(&room_key.clone()) {

        room_chat_set.entry(room_key).or_insert(vec![]);
        let room_key = &state.current_room.clone();

        // Add welcome message
        let msgs = state.room_chats.entry(room_key.clone()).or_default();
        msgs.push(format!("👋 Welcome to the {} room!", &room_key));
    }

    // Page layout
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(65),
            Constraint::Percentage(20),
        ],
    )
    .split(frame.area());

    // Center Nav Bar
    let centered_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(35), 
            Constraint::Percentage(65),
        ],
    )
    .split(main_layout[0]);

    // Nav Bar
    frame.render_widget(Tabs::new(vec!["Global <1>", "Rooms <2>", "Direct Messages <3>"])
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .select(1)
    , centered_layout[1]);


    // Messages display
    let input_str: &str = &state.input;
    let room_key = state.current_room.clone();
    let msgs = &state.room_chats;
    let message_str: Vec<String> = msgs[&room_key].iter().map(|m| format!("{}", m)).collect();
    let concatenated_messages = message_str.join("\n");
    let messages = Paragraph::new(concatenated_messages)
        .block(
            Block::bordered()
                .title(format!("Chatting in room {}", state.current_room))
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
                    KeyCode::Char('1') => {
                        state.tab = 0;
                    }
                    KeyCode::Char('2') => {
                        state.tab = 1;
                    }
                    KeyCode::Char('3') => {
                        state.tab = 2;
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

                            let msgs = state.room_chats.entry(room_key.clone()).or_default();
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