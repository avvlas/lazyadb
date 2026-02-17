use crossterm::event::KeyEvent;

use crate::action::Action;
use crate::app::App;
use crate::config::Config;

pub fn handle_key_event(app: &mut App, key: KeyEvent, config: &Config) {
    // Modal override: when help is shown, only allow closing it
    if app.show_help {
        if let Some(Action::CloseModal | Action::ToggleHelp) = lookup_action(key, app, config) {
            app.close_modal()
        }
        return;
    }

    if let Some(action) = lookup_action(key, app, config) {
        dispatch_action(app, action);
    }
}

fn lookup_action(key: KeyEvent, app: &App, config: &Config) -> Option<Action> {
    let key_seq = vec![key];
    config
        .keybindings
        .0
        .get(&app.focus)
        .and_then(|bindings| bindings.get(&key_seq))
        .cloned()
}

fn dispatch_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.quit(),
        Action::CycleFocus => app.cycle_focus(),
        Action::ToggleHelp => app.toggle_help(),
        Action::CloseModal => app.close_modal(),
        Action::DeviceListUp => app.select_prev_device(),
        Action::DeviceListDown => app.select_next_device(),
        Action::RefreshDevices => app.refresh_devices(),
        Action::EmulatorListUp => app.select_prev_emulator(),
        Action::EmulatorListDown => app.select_next_emulator(),
        Action::KillEmulator => app.kill_selected_emulator(),
        Action::EmulatorSelect => app.select_emulator(),
        Action::Suspend => { /* TODO: handle suspend if needed */ }
        _ => {}
    }
}
