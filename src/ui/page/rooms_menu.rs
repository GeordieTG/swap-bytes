use tokio::io;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};

use crate::{network::network::Client, state::STATE};

pub fn render(frame: &mut Frame) {

    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(65),
            Constraint::Percentage(20),
        ],
    )
    .split(frame.area());

    let centered_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(35), // left padding
            Constraint::Percentage(65), // center part for tabs
        ],
    )
    .split(main_layout[0]);


    // Tabs
    frame.render_widget(Tabs::new(vec!["Global", "Rooms", "Direct Messages"])
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .select(1)
    , centered_layout[1]);


    // Room list display
    let state = STATE.lock().unwrap();

    let room_items: Vec<ListItem> = state.rooms.iter().map(|room| ListItem::new(room.as_str())).collect();
    let rooms = List::new(room_items)
        .block(Block::bordered().title("📚 Select Room to Enter"))
        .highlight_style(Style::default().fg(Color::Yellow));
 
    // Create room option
    let input_str: &str = &state.input;
    let create_room = Paragraph::new(input_str)
    .block(
        Block::bordered()
            .title("Type new room name | Create new room <Right Arrow>")
            .style(Style::default().fg(Color::Blue))
    )
    .style(Style::default().fg(Color::White));

    frame.render_stateful_widget(rooms, main_layout[1], &mut state.room_list_state.clone());
    frame.render_widget(create_room, main_layout[2]);
}


pub async fn handle_events(client: &mut Client) -> io::Result<bool> {

    client.fetch_rooms().await;

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Tab => {
                        let mut state = STATE.lock().unwrap();
                        state.tab = 2;
                        state.input = String::new();
                    }
                    KeyCode::Right => {
                        let mut state = STATE.lock().unwrap();
                        if state.input != String::new() && !state.rooms.contains(&state.input) {
                            client.create_room(state.input.to_string()).await;
                            state.input = String::new();
                        }
                    }
                    KeyCode::Backspace => {
                        let mut state = STATE.lock().unwrap();
                        state.input.pop();
                    }
                    KeyCode::Down => {
                        let mut state = STATE.lock().unwrap();
                        if state.room_list_state.selected().unwrap() != state.rooms.len() - 1 {
                            state.room_list_state.select_next();
                        }
                    }
                    KeyCode::Up => {
                        let mut state = STATE.lock().unwrap();
                        state.room_list_state.select_previous();
                    }
                    KeyCode::Char(c) => {
                        let mut state = STATE.lock().unwrap();
                        state.input.push(c)
                    }
                    KeyCode::Enter => {
                        let mut state = STATE.lock().unwrap();
                        state.current_room = state.rooms.get(state.room_list_state.selected().expect("")).expect("").to_string();
                        state.tab = 4;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}