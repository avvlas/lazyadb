use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => app.close_modal(),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Tab => app.cycle_focus(),
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Esc => app.close_modal(),
        _ => {}
    }
}
