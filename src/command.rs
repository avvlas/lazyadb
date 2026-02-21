use crate::panes::Pane;

pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    DisconnectDevice(String),
    Focus(Pane),
}
