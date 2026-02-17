pub mod device_list;
pub mod emulator_list;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, FocusPanel};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Vertical: title bar | middle | command bar
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    draw_title_bar(frame, vertical[0]);

    // Middle: sidebar (20%) | content (80%)
    let middle = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(vertical[1]);

    // Sidebar: devices (top 50%) | emulators (bottom 50%)
    let sidebar =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(middle[0]);

    device_list::draw(frame, sidebar[0], app);
    emulator_list::draw(frame, sidebar[1], app);
    draw_content_area(frame, middle[1], matches!(app.focus, FocusPanel::Content));
    draw_command_bar(frame, vertical[2], app);

    if app.show_help {
        draw_help_overlay(frame, area);
    }
}

fn draw_title_bar(frame: &mut Frame, area: Rect) {
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "LazyADB v0.1.0",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let device_span = Line::from(vec![Span::styled(
        "device: <none>",
        Style::default().fg(Color::DarkGray),
    )]);

    let device = Paragraph::new(device_span).right_aligned();

    frame.render_widget(title, columns[0]);
    frame.render_widget(device, columns[1]);
}

fn draw_content_area(frame: &mut Frame, area: Rect, focused: bool) {
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

fn draw_command_bar(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::horizontal([Constraint::Min(0), Constraint::Length(8)]).split(area);

    // Left: keymap hints
    let mut hints = vec![("q", "Quit"), ("Tab", "Focus"), ("j/k", "Select")];
    if matches!(app.focus, FocusPanel::Emulators) {
        hints.push(("Enter", "Start"));
        hints.push(("x", "Kill"));
    }
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::White),
        ));
    }
    let left = Paragraph::new(Line::from(spans));
    frame.render_widget(left, columns[0]);

    // Right: help hint
    let right = Paragraph::new(Line::from(vec![Span::styled(
        "? help",
        Style::default().fg(Color::DarkGray),
    )]))
    .right_aligned();
    frame.render_widget(right, columns[1]);
}

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
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
