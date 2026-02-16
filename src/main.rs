use std::io;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let terminal = ratatui::init();
    let result = run(terminal);

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(draw)?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "LazyADB v0.1.0",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]))
    .centered()
    .block(Block::default());

    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "Press 'q' to quit",
        Style::default().fg(Color::DarkGray),
    )]))
    .centered()
    .block(Block::default().borders(Borders::NONE));

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" LazyADB ")
            .style(Style::default().fg(Color::Green)),
        area,
    );
    frame.render_widget(title, layout[1]);
    frame.render_widget(hint, layout[2]);
}
