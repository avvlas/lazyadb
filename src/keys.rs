use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, FocusPanel};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => app.close_modal(),
            _ => {}
        }
        return;
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            app.quit();
            return;
        }
        KeyCode::Tab => {
            app.cycle_focus();
            return;
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            return;
        }
        KeyCode::Esc => {
            app.close_modal();
            return;
        }
        _ => {}
    }

    // Panel-specific keys
    if matches!(app.focus, FocusPanel::DeviceList) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.select_next_device(),
            KeyCode::Char('k') | KeyCode::Up => app.select_prev_device(),
            KeyCode::Enter => app.confirm_device_selection(),
            _ => {}
        }
    }
}
