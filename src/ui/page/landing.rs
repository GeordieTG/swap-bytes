use std::io::Stdout;
use std::error::Error;

use crate::{state::STATE, ui::components::{input_component, title_component}};
use ratatui::{
    crossterm::event::{self, Event, KeyCode}, layout::{Constraint, Direction, Layout}, prelude::{CrosstermBackend, Frame}, style::Stylize, text::Line, Terminal
};

#[derive(Default)]
pub struct Landing {
    input: String
}


impl Landing {

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<bool, Box<dyn Error>> {
        terminal.draw(|f| self.ui(f))?;
        let should_quit = &self.handle_events().await?;
        Ok(*should_quit)
    }

    fn ui(&self, frame: &mut Frame) {
        
        // Ratatui page layout
        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(75),
                Constraint::Percentage(20),
            ],
        )
        .split(frame.area());
            
        // Swapbytes title
        let text: Vec<Line> = vec!["Welcome to".white().into(), "SwapBytes".blue().into()];
        let title = title_component(text);

        // Nickname entry
        let input_display = input_component(&self.input, "Enter Nickname | <Enter> to confirm".to_string());
    
        // Render
        frame.render_widget(title, layout[1]);
        frame.render_widget(input_display, layout[2]);
    
    }
    
    
    async fn handle_events(&mut self) -> Result<bool, std::io::Error> {
    
        let mut state = STATE.lock().unwrap();
    
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {

                        KeyCode::Char('q') => return Ok(true),

                        KeyCode::Backspace => {
                            self.input.pop();
                        }

                        KeyCode::Char(c) => {
                            self.input.push(c)
                        }

                        KeyCode::Enter => {
                            if !self.input.is_empty() {
                                state.nickname = self.input.to_string();
                                self.input = String::new();
                                return Ok(true)
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
        Ok(false)
    }
}