use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::command::Command;
use crate::message::Action;
use crate::panes::Pane;
use crate::state::{ModalState, State};

pub fn update(state: &mut State, action: &Action) -> Vec<Command> {
    match action {
        Action::OpenEmulators => {
            state.modal = ModalState::Emulators;
        }
        Action::EmulatorsUpdated(emulators) => {
            state.emulators.items = emulators.clone();
            if !state.emulators.items.is_empty() {
                state.emulators.selected_index = state
                    .emulators
                    .selected_index
                    .min(state.emulators.items.len() - 1);
            } else {
                state.emulators.selected_index = 0;
            }
        }
        Action::EmulatorListUp => {
            if matches!(state.modal, ModalState::Emulators) {
                state.emulators.selected_index =
                    state.emulators.selected_index.saturating_sub(1);
            }
        }
        Action::EmulatorListDown => {
            if matches!(state.modal, ModalState::Emulators) && !state.emulators.items.is_empty() {
                state.emulators.selected_index =
                    (state.emulators.selected_index + 1).min(state.emulators.items.len() - 1);
            }
        }
        Action::KillEmulator => {
            if matches!(state.modal, ModalState::Emulators) {
                if let Some(avd) = state.emulators.items.get(state.emulators.selected_index)
                    && let Some(serial) = &avd.running_serial
                {
                    return vec![Command::KillEmulator(serial.clone())];
                }
            }
        }
        Action::EmulatorSelect => {
            if matches!(state.modal, ModalState::Emulators) {
                if let Some(avd) = state.emulators.items.get(state.emulators.selected_index) {
                    if avd.is_running() {
                        state.modal = ModalState::None;
                        return vec![Command::Focus(Pane::Content)];
                    } else {
                        return vec![Command::StartEmulator(avd.name.clone())];
                    }
                }
            }
        }
        _ => {}
    }
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    if !matches!(state.modal, ModalState::Emulators) {
        return;
    }

    let rect = centered_rect(50, 50, area);
    frame.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" EMULATORS ")
        .border_style(Style::default().fg(Color::Green));

    if state.emulators.items.is_empty() {
        let paragraph = Paragraph::new("(no AVDs)").block(block);
        frame.render_widget(paragraph, rect);
        return;
    }

    let items: Vec<ListItem> = state
        .emulators
        .items
        .iter()
        .map(|avd| {
            let (icon, icon_color) = if avd.is_running() {
                ("▶", Color::Green)
            } else {
                ("■", Color::DarkGray)
            };

            let suffix = if avd.is_running() { " (running)" } else { "" };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::raw(format!("{}{}", avd.display_name(), suffix)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut list_state = ListState::default().with_selected(Some(state.emulators.selected_index));
    frame.render_stateful_widget(list, rect, &mut list_state);
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
