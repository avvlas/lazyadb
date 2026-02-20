use crate::action::Action;

pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    SendAction(Action),
}
