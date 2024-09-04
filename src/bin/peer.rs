use swapbytes::ui::page::landing::Landing;
use swapbytes::{network::network, ui::router::Router};
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
/// Logs can be found in "peer.log"
fn setup_logger() -> Result<(), Box<dyn Error>> {

    let log_file = File::create("src/log/peer.log")?;
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
    let mut tab_manager = Router::default();
    let mut landing = Landing::default();

    // Libp2p Setup
    let (mut client, event_loop) = network::new()?;    

    // Page to enter nickname
    while !landing.run(&mut terminal).await? {}

    // Enter the Application
    spawn(event_loop.run(client.clone()));
    
    // Main UI Loop
    while !tab_manager.run(&mut terminal, &mut client).await? {}

    // UI Clean Up
    disable_raw_mode()?;
    std_out::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}