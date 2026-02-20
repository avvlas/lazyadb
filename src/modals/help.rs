use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::command::Command;
use crate::state::State;
use crate::{message::Msg, state::ModalState};

pub fn update(state: &mut State, action: &Msg) -> Vec<Command> {
    match action {
        Msg::ToggleHelp => {
            if matches!(state.modal, ModalState::None) {
                state.modal = ModalState::Help;
            } else {
                state.modal = ModalState::None;
            }
        }
        Msg::CloseModal => {
            state.modal = ModalState::None;
        }
        _ => {}
    }
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    if !matches!(state.modal, ModalState::Help) {
        return;
    }

    let rect = centered_rect(50, 50, area);
    frame.render_widget(Clear, rect);

    let help_text = "\
Keybindings
───────────
q         Quit
Tab       Cycle focus (Devices → Emulators → Content)
j / ↓     Select next item
k / ↑     Select previous item
x         Kill running emulator (Emulators panel)
?         Toggle help
Esc       Close modal";

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" HELP ")
        .border_style(Style::default().fg(Color::Green));
    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, rect);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
