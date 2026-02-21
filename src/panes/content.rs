use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::message::Action;
use crate::panes::Pane;
use crate::command::Command;
use crate::state::State;

pub fn update(_state: &mut State, _action: &Action) -> Vec<Command> {
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus == Pane::Content;
    let border_color = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" CONTENT ")
        .border_style(Style::default().fg(border_color));
    let paragraph = Paragraph::new("Select a device to begin").block(block);
    frame.render_widget(paragraph, area);
}
