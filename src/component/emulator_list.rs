use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::{App, FocusPanel};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let focused = matches!(app.focus, FocusPanel::Emulators);
    let border_color = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" EMULATORS ")
        .border_style(Style::default().fg(border_color));

    if app.emulators.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new("(no AVDs)").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .emulators
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

    let mut state = ListState::default().with_selected(Some(app.selected_emulator_index));
    frame.render_stateful_widget(list, area, &mut state);
}
