use crate::panes::Pane;

pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    Focus(Pane),
}
