use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, area: Rect, focused: bool) {
    let border_color = if focused { Color::Green } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DEVICES ")
        .border_style(Style::default().fg(border_color));
    let paragraph = Paragraph::new("(no devices)").block(block);
    frame.render_widget(paragraph, area);
}
