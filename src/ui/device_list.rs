use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::adb::device::DeviceState;
use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.focus, crate::app::FocusPanel::DeviceList);
    let border_color = if focused { Color::Green } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DEVICES ")
        .border_style(Style::default().fg(border_color));

    if app.devices.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new("(no devices)").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .devices
        .iter()
        .map(|device| {
            let (icon, icon_color) = match device.state {
                DeviceState::Online => ("●", Color::Green),
                DeviceState::Offline => ("○", Color::Red),
                DeviceState::Unauthorized => ("⚠", Color::Yellow),
                DeviceState::Unknown(_) => ("?", Color::DarkGray),
            };

            let is_active = app
                .active_device_serial
                .as_ref()
                .is_some_and(|s| s == &device.serial);

            let name = device.display_name();
            let active_marker = if is_active { " *" } else { "" };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::raw(format!("{}{}", name, active_marker)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.selected_device_index));
    frame.render_stateful_widget(list, area, &mut state);
}
