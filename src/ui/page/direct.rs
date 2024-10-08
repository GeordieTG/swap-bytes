use std::rc::Rc;
use crate::{network::client::Client, state::STATE, ui::components::{input_component, list_component}};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::*,
};

/// Represents the currently selected section.
#[derive(Default, PartialEq)]
enum Section {
    #[default]
    None,
    Request,
    Response
}


/// A page for users to request and send files with all other peers on the network.
#[derive(Default)]
pub struct Direct {
    input: String,
    peer_list_state: ListState,
    request_list_state: ListState,
    selected_section: Section,
    popup: Section
}

impl Direct {

    pub fn render(&mut self, frame: &mut Frame, layout: Rc<[Rect]>) {
    
        // Allows to split the screen to have both Request and Received lists.
        let horizontal_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ],
        )
        .split(layout[1]);

        // Request a file section
        let peer_items = self.format_peers();
        let peers_display = list_component(peer_items, "🌍 Request File".to_string());
        frame.render_stateful_widget(peers_display, horizontal_layout[0], &mut self.peer_list_state.clone());
        

        // Incoming Requests section
        let request_items = self.format_requests();
        let requests_display = list_component(request_items, "🚀 Incoming Request".to_string());
        frame.render_stateful_widget(requests_display, horizontal_layout[1], &mut self.request_list_state.clone());
    
    
        // Display the input for request messages and response file paths when required
        match self.popup  {
            Section::Request => {
                let popup = input_component(&self.input, "Request a file | <Enter> to send".to_string());
                frame.render_widget(popup, layout[2]);
            }
            Section::Response => {
                let popup = input_component(&self.input, "Enter a File Path | <Enter> to send".to_string());
                frame.render_widget(popup, layout[2]);
            }
            Section::None => {}
        }
    }
    
    
    /// Handles key stroke events for the file sharing page.
    pub async fn handle_events(&mut self, client: &mut Client, key: KeyEvent) {
        
        match key.code {

            // User input into the message box
            KeyCode::Char(c) => {
                self.input.push(c);
            }

            // Allows for deletion of characters in the message box
            KeyCode::Backspace => {
                self.input.pop();
            }

            // Moves down the currently selected list
            KeyCode::Down => {
                if self.popup == Section::None {
                    match self.selected_section {
                        Section::Request => self.peer_list_state.select_next(),
                        Section::Response => self.request_list_state.select_next(),
                        Section::None => {}
                    }
                }
            }

            // Moves up the currently selected list
            KeyCode::Up => {
                if self.popup == Section::None {
                    match self.selected_section {
                        Section::Request => self.peer_list_state.select_previous(),
                        Section::Response => self.request_list_state.select_previous(),
                        Section::None => {}
                    }
                }
            }

            // Selects the "Send Request" section
            KeyCode::Left => {
                if self.popup == Section::None {
                    self.selected_section = Section::Request;
                    self.request_list_state.select(None);
                    self.peer_list_state.select_first();
                }
            }

            // Selects the "Incoming Requests" section
            KeyCode::Right => {
                if self.popup == Section::None {
                    self.selected_section = Section::Response;
                    self.peer_list_state.select(None);
                    self.request_list_state.select_first();
                }
            }

            // Handles confirmation of the current popup
            KeyCode::Enter => {
                match self.selected_section {
                    Section::Request => self.handle_requests(client).await,
                    Section::Response => self.handle_response(client).await,
                    Section::None => {}
                }
            }

            _ => {}
        }
    }


    /// Fetches connected peers from the global store and formats them in a way to be displayed in the Ratatui UI.
    fn format_peers(&self) -> Vec<ListItem> {

        let state = STATE.lock().unwrap();

        let peers: Vec<ListItem> = state
        .peers
        .iter()
        .filter_map(|peer_id| {
            state.nicknames.get(&peer_id.to_string()).map(|nickname| ListItem::new(format!("{}", nickname.clone())))
        })
        .collect();

        peers
    }


    /// Fetches current incoming requests from the global store and formats them in a way to be displayed in the Ratatui UI.
    fn format_requests(&self) -> Vec<ListItem>  {

        let state = STATE.lock().unwrap();
        
        let request_items: Vec<ListItem> = state
            .requests.iter()
            .map(|request| ListItem::new(format!("{} - {}", state.nicknames.get(&request.0.to_string().clone()).expect(""), request.1)))
            .collect();  

        request_items
    }


    /// Handles events in the "Request a File" section.
    /// If a user is selected and the request popup is not already showing, the request input popup will be displayed.
    /// Otherwise if it is already showing, the request with the message typed into the input will be sent to the selected user.
    async fn handle_requests(&mut self, client: &mut Client) {

        let state = STATE.lock().unwrap();

        if let Some(selected_index) = self.peer_list_state.selected() {
            if state.peers.len() > 0 {
                if self.popup != Section::Request {
                    self.popup = Section::Request;
                } else if let Some(selected_user) = state.peers.get(selected_index) {
                    client.send_request(self.input.clone(), *selected_user).await;
                    self.reset_popup();
                }
            }
        }
    }


    /// Handles events in the "Incoming Requests" section.
    /// If a user is selected and the response popup is not already showing, the response input popup will be displayed.
    /// Otherwise if it is already showing, the response with the file at the given path will be sent to the selected user.
    async fn handle_response(&mut self, client: &mut Client) {

        let mut state = STATE.lock().unwrap();

        if let Some(selected_index) = self.request_list_state.selected() {
            if state.requests.len() > 0 {
                if self.popup != Section::Response {
                    self.popup = Section::Response;
                } else {
                    let (_, _, channel) = state.requests.remove(selected_index);
                    client.send_response("swapbytes.txt".to_string(), self.input.to_string(), channel).await;
                    self.reset_popup();
                }
            }
        }
    }

    /// Clears the currently shown Popup
    fn reset_popup(&mut self) {
        self.popup = Section::None;
        self.input.clear();
    }

}