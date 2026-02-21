use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{message::Action, panes::Pane};
use crate::command::Command;
use crate::state::State;

pub fn update(state: &mut State, action: &Action) -> Vec<Command> {
    match action {
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
            state.emulators.selected_index = state.emulators.selected_index.saturating_sub(1);
        }
        Action::EmulatorListDown => {
            if !state.emulators.items.is_empty() {
                state.emulators.selected_index =
                    (state.emulators.selected_index + 1).min(state.emulators.items.len() - 1);
            }
        }
        Action::KillEmulator => {
            if let Some(avd) = state.emulators.items.get(state.emulators.selected_index)
                && let Some(serial) = &avd.running_serial
            {
                return vec![Command::KillEmulator(serial.clone())];
            }
        }
        Action::EmulatorSelect => {
            if let Some(avd) = state.emulators.items.get(state.emulators.selected_index) {
                if avd.is_running() {
                    return vec![Command::Focus(Pane::Content)];
                } else {
                    return vec![Command::StartEmulator(avd.name.clone())];
                }
            }
        }
        _ => {}
    }
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus == Pane::Emulators;
    let border_color = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" EMULATORS ")
        .border_style(Style::default().fg(border_color));

    if state.emulators.items.is_empty() {
        let paragraph = Paragraph::new("(no AVDs)").block(block);
        frame.render_widget(paragraph, area);
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
    frame.render_stateful_widget(list, area, &mut list_state);
}
