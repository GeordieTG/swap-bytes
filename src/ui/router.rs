use crate::network::network::Client;
use crate::state::STATE;
use crate::ui::page::global;
use crate::ui::page::direct;
use crate::ui::page::rating;
use crate::ui::page::rooms_menu;
use crate::ui::page::room;
use crate::ui::page::landing;
use ratatui::prelude::*;


/// Displays the corresponding page in the UI using Ratatui depending on which tab is selected.
/// (Global Chat, Rooms, Direct Chat).
pub fn ui(
    frame: &mut Frame,
) {

    let state = STATE.lock().unwrap();
    let tab = state.tab;
    drop(state);
    
    match tab {
        0 => global::render(frame),
        1 => rooms_menu::render(frame),
        2 => direct::render(frame),
        3 => rating::render(frame),
        4 => room::render(frame),  
        5 => landing::render(frame),
        _ => { log::info!("Goofed")}
    }
}


/// Handles Key Presses to modify the UI
/// Eg. Typing a message, Tab changes, Arrows to select various options
/// Takes various state as input (selected tabs, messages array, list states etc).
pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {

    let state = STATE.lock().unwrap();
    let tab = state.tab;
    drop(state);

    match tab {
        0 => global::handle_events(client).await,
        1 => rooms_menu::handle_events(client).await,
        2 => direct::handle_events(client).await,  
        3 => rating::handle_events(client).await,  
        4 => room::handle_events(client).await,
        5 => landing::handle_events().await,
        _ => {Ok(false)}
    }
}