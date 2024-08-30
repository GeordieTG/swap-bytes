use ratatui::{style::{Color, Style}, text::Line, widgets::{Block, List, ListItem, Paragraph}};
use tui_big_text::{BigText, PixelSize};
use strum_macros::EnumIter;

// List of all application tabs.
#[derive(Default, Debug, EnumIter, PartialEq, Clone)]
pub enum Tab {
    #[default]
    Chat,
    RoomMenu,
    Direct,
    Rating
}


impl Tab {
    // Switch to the next tab in the cycle.
    pub fn next(&self) -> Tab {
        match self {
            Tab::Chat => Tab::RoomMenu,
            Tab::RoomMenu => Tab::Direct,
            Tab::Direct => Tab::Chat,
            _ => {Tab::Chat}
        }
    }
}


pub fn message_component(messages: &str, title: String) -> Paragraph {
    
    let messages = Paragraph::new(messages)
            .block(
                Block::bordered()
                    .style(Style::default().fg(Color::White)).title(title)
            )
            .style(Style::default().fg(Color::White));

    messages
}

pub fn input_component(input_str: &str, title: String) -> Paragraph{

    let input = Paragraph::new(input_str)
        .block(
            Block::bordered()
                .title(title)
                .style(Style::default().fg(Color::Blue))
        )
        .style(Style::default().fg(Color::White));

    input
}


pub fn list_component(items: Vec<ListItem>, title: String) -> List {

    let list = List::new(items)
            .block(Block::bordered().title(title))
            .highlight_style(Style::default().fg(Color::Yellow));

    list
}


pub fn title_component(text: Vec<Line>) -> BigText {

    let title = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .alignment(ratatui::layout::Alignment::Center)
            .centered()
            .lines(text.clone())
            .build();

    title
}