use crate::components::panes::Pane;

#[allow(dead_code)]
pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    OpenEmulatorsModal,
    CloseEmulatorsModal,

    RefreshDevices,
    DisconnectDevice(String),

    Focus(Pane),
}
