use crate::app::Pane;

pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    Focus(Pane),
}
