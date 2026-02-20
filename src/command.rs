use crate::app::FocusPanel;

pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    Focus(FocusPanel),
}
