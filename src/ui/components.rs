use std::rc::Rc;

use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize}, widgets::Tabs, Frame};

pub fn navbar(frame: &mut Frame) -> Rc<[Rect]> {

    // Page layout
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(65),
            Constraint::Percentage(20),
        ],
    )
    .split(frame.area());

    // Center Nav Bar
    let centered_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(35), 
            Constraint::Percentage(65),
        ],
    )
    .split(main_layout[0]);

    // Nav Bar
    frame.render_widget(Tabs::new(vec!["Global", "Rooms", "Direct Messages"])
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .select(0)
    ,centered_layout[1]);

    main_layout
}

