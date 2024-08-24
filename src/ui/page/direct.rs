use tokio::io;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::*,
    widgets::*,
};

use crate::{network::network::Client, state::STATE};

use derive_setters::Setters;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};


#[derive(Debug, Default, Setters)]
struct Popup<'a> {
    #[setters(into)]
    title: Line<'a>,
    #[setters(into)]
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}


impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ensure that all cells under the popup are cleared to avoid leaking content
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}

static mut SHOW_REQUEST_POPUP: bool = false;
static mut SHOW_RESPONSE_POPUP: bool = false;
static mut SELECTED_SECTION: usize = 0;

pub fn render(frame: &mut Frame) {

    let mut state = STATE.lock().unwrap();

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

    let horizontal_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ],
    )
    .split(main_layout[1]);


    // Tabs
    frame.render_widget(Tabs::new(vec!["Global", "Rooms", "Direct Messages"])
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .select(2)
    , centered_layout[1]);


    // Request a file
    let peers = state.peers.lock().unwrap();
    let peer_list: Vec<String> = peers.iter().map(|peer_id| format!("{}", peer_id)).collect();  

    let peer_items: Vec<ListItem> = peer_list
    .iter()
    .filter_map(|peer| {
        state.nicknames.get(peer).map(|nickname| ListItem::new(format!("{}", nickname)))
    })

    .collect();
    drop(peers);

    let peers = List::new(peer_items)
        .block(Block::bordered().title("🌍 Request File"))
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_stateful_widget(peers, horizontal_layout[0], &mut state.peer_list_state);



    // Incoming Requests
    let incoming_requests = &state.requests;
    let request_items: Vec<String> = incoming_requests.iter().map(|request| format!("{} - {}", state.nicknames.get(&request.0.to_string()).expect(""), request.1)).collect();  
    let requests = List::new(request_items)
        .block(Block::bordered().title("🚀 Incoming Request"))
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_stateful_widget(requests, horizontal_layout[1], &mut state.request_list_state);


    if unsafe { SHOW_REQUEST_POPUP } {
        let popup = Popup::default()
        .content(state.input.clone())
        .title("Request a file")
        .title_style(Style::new().white().bold())
        .border_style(Style::new().red());
        frame.render_widget(popup, main_layout[2]);
    }

    if unsafe { SHOW_RESPONSE_POPUP } {
        let popup = Popup::default()
        .content(state.input.clone())
        .title("Enter a File Path to Send")
        .title_style(Style::new().white().bold())
        .border_style(Style::new().red());
        frame.render_widget(popup, main_layout[2]);
    }
}


/// Handles key stroke events for the direct messages page.
pub async fn handle_events(client: &mut Client) -> io::Result<bool> {

    let mut state = STATE.lock().unwrap();

    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Tab => {
                        state.tab = 0;
                    }
                    KeyCode::Char(c) => {
                        state.input.push(c);
                    }
                    KeyCode::Backspace => {
                        state.input.pop();
                    }
                    KeyCode::Down => {

                        if unsafe { SELECTED_SECTION } == 0 {
                            state.peer_list_state.select_next();
                        } else {
                            state.request_list_state.select_next();
                        }
                    }
                    KeyCode::Up => {
                        if unsafe { SELECTED_SECTION } == 0 {
                            state.peer_list_state.select_previous();
                        } else {
                            state.request_list_state.select_previous();
                        }
                    }
                    KeyCode::Left => {
                        unsafe { SELECTED_SECTION = 0 };
                        state.request_list_state.select(None);
                        state.peer_list_state.select(Some(0));
                    }
                    KeyCode::Right => {
                        unsafe { SELECTED_SECTION = 1 };
                        state.peer_list_state.select(None);
                        state.request_list_state.select(Some(0));
                    }
                    KeyCode::Enter => {

                        // Request File Section
                        if unsafe { SELECTED_SECTION } == 0 {
                            if state.peer_list_state.selected() != None && unsafe { SHOW_REQUEST_POPUP } == false {
                                unsafe { SHOW_REQUEST_POPUP = true };
                            } else if unsafe { SHOW_REQUEST_POPUP } == true {
                                log::info!("Gonna send the request");
                                let peers = state.peers.lock().unwrap();
                                let selected_user = peers.get(state.peer_list_state.selected().expect(""))
                                    .expect("Peer not found in the list");
                                client.send_request(state.input.clone(), *selected_user).await;
                                unsafe { SHOW_REQUEST_POPUP = false };
                                drop(peers);
                                state.input.clear();
                            }
                        } else {
                            // Incoming Request Section
                            if state.request_list_state.selected() != None && unsafe { SHOW_RESPONSE_POPUP } == false {
                                unsafe { SHOW_RESPONSE_POPUP = true };
                            } else if unsafe { SHOW_RESPONSE_POPUP } == true {
                                log::info!("Gonna send the response");
                                let index = state.request_list_state.selected().expect("");
                                let request = state.requests.remove(index);
                                let channel = request.2; // Move the channel out

                                client.send_response("swapbytes.txt".to_string(), state.input.to_string(), channel).await;
                                unsafe { SHOW_RESPONSE_POPUP = false };
                                state.input.clear();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}