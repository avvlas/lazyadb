use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::action::Action;
use crate::adb::device::{ConnectionType, Device, DeviceState};
use crate::app::FocusPanel;
use crate::command::Command;
use crate::state::State;

fn physical_devices(devices: &[Device]) -> Vec<&Device> {
    devices
        .iter()
        .filter(|d| d.connection_type != ConnectionType::Emulator)
        .collect()
}

pub fn update(state: &mut State, action: &Action) -> Vec<Command> {
    match action {
        Action::DevicesUpdated(devices) => {
            state.devices.items = devices.clone();
            let count = physical_devices(&state.devices.items).len();
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
            let count = physical_devices(&state.devices.items).len();
            if count > 0 {
                state.devices.selected_index =
                    (state.devices.selected_index + 1).min(count - 1);
            }
        }
        _ => {}
    }
    Vec::new()
}

pub fn draw(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus == FocusPanel::DeviceList;
    let border_color = if focused { Color::Green } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DEVICES ")
        .border_style(Style::default().fg(border_color));

    let physical = physical_devices(&state.devices.items);

    if physical.is_empty() {
        let paragraph = Paragraph::new("(no devices)").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = physical
        .iter()
        .map(|device| {
            let (icon, icon_color) = match device.state {
                DeviceState::Online => ("●", Color::Green),
                DeviceState::Offline => ("○", Color::Red),
                DeviceState::Unauthorized => ("⚠", Color::Yellow),
                DeviceState::Unknown(_) => ("?", Color::DarkGray),
            };

            let name = device.display_name();

            let line = Line::from(vec![
                Span::styled(icon.to_string(), Style::default().fg(icon_color)),
                Span::raw(name.to_string()),
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
