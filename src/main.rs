pub mod network {
    pub mod network;
    pub mod mdns;
    pub mod gossipsub;
    pub mod request_response;
    pub mod kademlia;
}
pub mod router;
pub mod types;
pub mod state;
pub mod ui {
    pub mod global_ui;
    pub mod rooms_ui;
    pub mod direct_ui;
    pub mod room;
    pub mod landing;
    pub mod rating_ui;
}
use std::io as std_out;
use tokio::spawn;
use std::error::Error;
use log::info;
use fern::Dispatch;
use chrono::Local;
use std::fs::File;
use ratatui::{
    crossterm::{
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    prelude::*,
};


/// Setup a global logger for the application. As Ratatui UI replaces the terminal, we need a place to safely log information.
/// Logs can be found in "app.log"
fn setup_logger() -> Result<(), Box<dyn Error>> {

    let log_file = File::create("app.log")?;
    Dispatch::new()
        .filter(|metadata| metadata.level() <= log::LevelFilter::Info)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .chain(log_file)
        .apply()?;

        info!("Application started");

    Ok(())
}


/// Main entry point of the application.
/// A chat and file sharing application designed for students to organise the trading of class notes.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    setup_logger().expect("Logger setup failed");

    // Ratatui UI Setup
    enable_raw_mode()?;
    std_out::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std_out::stdout()))?;


    // Libp2p Setup
    let (mut client, event_loop) = network::network::new()?;    

    // Page to enter nickname
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui::landing::render(f))?;
        should_quit = ui::landing::handle_events().await?;
    }

    // Enter the Application
    spawn(event_loop.run());
    
    // Main UI Loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| router::ui(f))?;
        should_quit = router::handle_events( &mut client).await?;
    }

    // UI Clean Up
    disable_raw_mode()?;
    std_out::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}