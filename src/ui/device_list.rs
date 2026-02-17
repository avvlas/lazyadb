use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::adb::device::DeviceState;
use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.focus, crate::app::FocusPanel::DeviceList);
    let border_color = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DEVICES ")
        .border_style(Style::default().fg(border_color));

    let physical = app.physical_devices();

    if physical.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new("(no devices)").block(block);
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
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::raw(format!("{}", name)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default().with_selected(Some(app.selected_device_index));
    frame.render_stateful_widget(list, area, &mut state);
}
