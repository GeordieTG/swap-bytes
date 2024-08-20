use crate::state::STATE;
use ratatui::{
    crossterm::event::{self, Event, KeyCode}, layout::{Constraint, Direction, Layout}, style::Color, widgets::{Block, Paragraph}
};

use ratatui::prelude::{Frame, Style, Stylize};
use tui_big_text::{BigText, PixelSize};

pub fn render(frame: &mut Frame) {

    let state = STATE.lock().unwrap();

    // Page layout
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(75),
            Constraint::Percentage(20),
        ],
    )
    .split(frame.area());

    // Messages display
    let input_str: &str = &state.input;
    
    // User input
    let input_display = Paragraph::new(input_str)
    .block(
        Block::bordered()
            .title("Enter Nickname | <Enter> to confirm")
            .style(Style::default().fg(Color::White))
    )
    .style(Style::default().fg(Color::White));

    let title = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .alignment(ratatui::layout::Alignment::Center)
        .centered()
        .lines(vec!["Welcome to".white().into(), "SwapBytes".blue().into()])
        .build();

    // Render
    frame.render_widget(title, main_layout[1]);
    frame.render_widget(input_display, main_layout[2]);

}


pub async fn handle_events() -> Result<bool, std::io::Error> {

    let mut state = STATE.lock().unwrap();

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Backspace => {
                        state.input.pop();
                    }
                    KeyCode::Char(c) => {
                        state.input.push(c)
                    }
                    KeyCode::Enter => {
                        state.nickname = state.input.to_string();
                        state.input = String::new();
                        state.tab = 0;
                        return Ok(true)
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}