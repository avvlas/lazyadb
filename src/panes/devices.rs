use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::adb::device::{ConnectionType, DeviceState};
use crate::command::Command;
use crate::message::Action;
use crate::panes::Pane;
use crate::state::State;

pub fn update(state: &mut State, action: &Action) -> Vec<Command> {
    match action {
        Action::DevicesUpdated(devices) => {
            state.devices.items = devices.clone();
            let count = state.devices.items.len();
            if count > 0 {
                state.devices.selected_index = state.devices.selected_index.min(count - 1);
            } else {
                state.devices.selected_index = 0;
            }
        }
        Action::DeviceListUp => {
            state.devices.selected_index = state.devices.selected_index.saturating_sub(1);
        }
        Action::DeviceListDown => {
            let count = state.devices.items.len();
            if count > 0 {
                state.devices.selected_index = (state.devices.selected_index + 1).min(count - 1);
            }
        }
        Action::DisconnectDevice => {
            if let Some(device) = state.devices.items.get(state.devices.selected_index) {
                match device.connection_type {
                    ConnectionType::Emulator => {
                        return vec![Command::KillEmulator(device.serial.clone())];
                    }
                    ConnectionType::Tcp => {
                        return vec![Command::DisconnectDevice(device.serial.clone())];
                    }
                    ConnectionType::Usb => {}
                }
            }
        }
        _ => {}
    }
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus == Pane::DeviceList;
    let border_color = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DEVICES ")
        .border_style(Style::default().fg(border_color));

    if state.devices.items.is_empty() {
        let paragraph = Paragraph::new("(no devices)").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = state
        .devices
        .items
        .iter()
        .map(|device| {
            let (icon, icon_color) = match device.state {
                DeviceState::Online => ("●", Color::Green),
                DeviceState::Offline => ("○", Color::Red),
                DeviceState::Unauthorized => ("⚠", Color::Yellow),
                DeviceState::Unknown(_) => ("?", Color::DarkGray),
            };

            let conn_tag = match device.connection_type {
                ConnectionType::Usb => " [USB]",
                ConnectionType::Tcp => " [TCP]",
                ConnectionType::Emulator => " [EMU]",
            };

            let name = device.display_name();

            let line = Line::from(vec![
                Span::styled(icon.to_string(), Style::default().fg(icon_color)),
                Span::raw(format!(" {}", name)),
                Span::styled(conn_tag, Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut list_state = ListState::default().with_selected(Some(state.devices.selected_index));
    frame.render_stateful_widget(list, area, &mut list_state);
}
