use crate::{network::network::Client, state::STATE, ui::components::navbar};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};

// This is the Ratatui UI for the Global Chat page.
pub fn render(frame: &mut Frame) {

    let state = STATE.lock().unwrap();

    let main_layout = navbar(frame);

    // Messages display
    let input_str: &str = &state.input;
    let msgs = state.messages.get("global").expect("Global Chat should be in the messages HashMap at all times");
    let message_str: Vec<String> = msgs.iter().map(|m| format!("{}", m)).collect();
    let concatenated_messages = message_str.join("\n");
    let messages = Paragraph::new(concatenated_messages)
        .block(
            Block::bordered()
                .style(Style::default().fg(Color::White)).title(state.nickname.clone())
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

    frame.render_widget(messages, main_layout[1]);
    frame.render_widget(input_display, main_layout[2]);

}


// This function handles all user input events for the Global Chat page.
pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {

    let mut state = STATE.lock().unwrap();
    
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {

                    KeyCode::Char('q') => return Ok(true),

                    KeyCode::Tab => {
                        state.tab = 1;
                        state.input = String::new();
                    }

                    KeyCode::Backspace => {
                        state.input.pop();
                    }

                    KeyCode::Char(c) => {
                        state.input.push(c)
                    }

                    KeyCode::Enter => {
                        {
                            let message = state.input.to_string().clone();
                            let msgs: &mut Vec<String> = &mut state.messages.get_mut("global").expect("");
                            msgs.push(format!("{}: {}", "You".to_string(), message));
                        }

                        client.send_message(state.input.to_string(), "global".to_string()).await;
                        state.input.clear();
                    }

                    _ => {}
                }
            }
        }
    }
    Ok(false)
}