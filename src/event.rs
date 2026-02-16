use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll_event() -> color_eyre::Result<AppEvent> {
    if event::poll(Duration::from_millis(16))?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        return Ok(AppEvent::Key(key));
    }
    Ok(AppEvent::Tick)
}
