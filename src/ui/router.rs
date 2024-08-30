use crate::network::client::Client;
use crate::state::STATE;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::Tabs;
use std::error::Error;
use std::io::Stdout;
use std::rc::Rc;
use ratatui::Terminal;
use strum::IntoEnumIterator;

use super::page::direct::Direct;
use super::page::chat::Chat;
use super::page::rating::Rating;
use super::page::rooms_menu::RoomMenu;
use super::components::Tab;


/// The main UI handler that renders pages depending on the current tab
#[derive(Default)]
pub struct Router {
    tab: Tab,
    room_menu: RoomMenu,
    global: Chat,
    direct: Direct,
    rating: Rating
}


impl Router {

    /// Function called by the main loop to continuously render the UI and listen for user input.
    /// Handle events will return a bool, and if true, indicates the shut down of the application.
    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>, client: &mut Client) -> Result<bool, Box<dyn Error>> {

        terminal.draw(|f| self.ui(f))?;
        let should_quit = &self.handle_events(client).await?;
        Ok(*should_quit)
    }


    /// Displays the corresponding page in the Ratatui UI depending on which tab is selected.
    fn ui(&mut self, frame: &mut Frame) {
        
        let layout = self.page_template(frame);
        
        match self.tab {
            Tab::Chat => self.global.render(frame, layout),
            Tab::RoomMenu => self.room_menu.render(frame, layout),
            Tab::Direct => self.direct.render(frame, layout),
            Tab::Rating => self.rating.render(frame, layout),
        }
    }


    /// Handles key presses to modify the UI, eg. typing a message, tab changes, arrows to select various options
    async fn handle_events(&mut self, client: &mut Client) -> Result<bool, std::io::Error> {

        let tab = self.tab.clone();

        if tab == Tab::RoomMenu {
            client.fetch_rooms().await;
        }

        if tab == Tab::Chat {
            let mut state = STATE.lock().unwrap();
            let room = state.current_room.clone();
            state.notifications.insert(room, false);
        }

        {
            let state = STATE.lock().unwrap();
            if state.current_rating.is_some() {
                self.tab = Tab::Rating;
            }
        }

        let switch_tab_callback = |new: Tab| self.tab = new;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {

                        // Handle application close (Command is global to all tabs)
                        KeyCode::Esc => return Ok(true),

                        // Handle a tab change (Command is global to all tabs)
                        KeyCode::Tab => {
                            self.tab = self.tab.next();
                        }

                        // Handle commands depending on the current tab                        
                        _ => match tab {
                            Tab::Chat => self.global.handle_events(client, key).await,
                            Tab::RoomMenu => self.room_menu.handle_events(client, key, switch_tab_callback).await,
                            Tab::Direct => self.direct.handle_events(client, key).await,
                            Tab::Rating => self.rating.handle_events(client, key, switch_tab_callback).await,
                        },
                    };
                }
            }
        }
        Ok(false)
    }

    /// Creates the main layout for the page and inserts the Navbar. 
    /// The remainder of the page will be injected with the current tab content.
    fn page_template(&mut self, frame: &mut Frame) -> Rc<[Rect]> {

        // Splits the page into thirds
        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(10), // Navbar
                Constraint::Percentage(65), // Main Display
                Constraint::Percentage(20), // Input if required
            ],
        )
        .split(frame.area());

        // Used to center the Navbar
        let centered_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(35), 
                Constraint::Percentage(65),
            ],
        )
        .split(main_layout[0]);

        // Curenntly selected tab so that we can highlight it in the Navbar
        let index = Tab::iter().position(|e| e == self.tab).unwrap();
        let (room_title, direct_title) = self.calculate_notifications();

        frame.render_widget(Tabs::new(vec!["Chat", &room_title, &direct_title])
        .style(Style::default().white())
        .highlight_style(Style::default().yellow())
        .select(index)
        , centered_layout[1]);

        main_layout
    }


    /// Calculate whether there are unseen messages / requests. Returns the appropriate Tab displays for Rooms and Direct Messages
    /// depending on whether there is a notification.
    fn calculate_notifications(&mut self) -> (String, String) {

        let state = STATE.lock().unwrap();

        let direct_notifications = state.requests.len() > 0;

        // Ensures notifications aren't shown if the user is currently looking at the chat we received a message for
        let room_notifications = state.notifications
        .iter()
        .any(|(room, &value)| (room != &state.current_room || self.tab != Tab::Chat) && value);

        let room_title = if room_notifications {"Rooms 🔔"} else {"Rooms"};
        let direct_title = if direct_notifications {"Direct Messages 🔔"} else {"Direct Messages"};

        (room_title.to_string(), direct_title.to_string())
    }

}