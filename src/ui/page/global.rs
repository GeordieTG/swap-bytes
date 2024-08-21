use crate::{network::network::Client, state::STATE};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};


pub fn render(frame: &mut Frame) {

    let state = STATE.lock().unwrap();

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
    .select(0)
    , centered_layout[1]);


    // Messages display
    let input_str: &str = &state.input;
    let msgs = state.messages.lock().unwrap();
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

    // Render
    frame.render_widget(messages, main_layout[1]);
    frame.render_widget(input_display, main_layout[2]);

}



pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {

    let mut state = STATE.lock().unwrap();

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
                            let mut msgs: std::sync::MutexGuard<Vec<String>> = state.messages.lock().unwrap();
                            msgs.push(format!("{}: {}", "You".to_string(), state.input.to_string()));
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