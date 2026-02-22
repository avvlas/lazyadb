use crate::adb::device::Device;
use crate::components::panes::Pane;

#[allow(dead_code)]
pub enum Command {
    StartEmulator(String),
    KillEmulator(String),
    OpenEmulatorsModal,
    CloseEmulatorsModal,

    RefreshDevices,
    RefreshDeviceInfo(String),
    DisconnectDevice(String),

    DeviceSelected(Option<Device>),
    Focus(Pane),
}
