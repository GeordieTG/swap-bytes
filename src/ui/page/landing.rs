use std::io::Stdout;
use std::error::Error;

use crate::{state::STATE, ui::components::{input_component, title_component}};
use ratatui::{
    crossterm::event::{self, Event, KeyCode}, layout::{Constraint, Direction, Layout}, prelude::{CrosstermBackend, Frame}, style::Stylize, text::Line, Terminal
};


/// A landing page where users will enter a nickname before entering the main application.
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

    /// Simply renders the page consisting of the Swapbytes title and an input field at the bottom of the page to allow the
    /// user to enter their nickname.
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
    
    
    /// Event handler for the Landing page. Listens for user keystrokes.
    async fn handle_events(&mut self) -> Result<bool, std::io::Error> {
    
        let mut state = STATE.lock().unwrap();
    
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {

                        // Handle application close
                        KeyCode::Esc => return Ok(true),

                        // Allows for deletion of characters in the input field
                        KeyCode::Backspace => {
                            self.input.pop();
                        }

                        // User input into the message box
                        KeyCode::Char(c) => {
                            self.input.push(c)
                        }

                        // Submits the nickname and will proceed to enter the main application
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