use strum::Display;

use crate::adb::device::Device;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Quit,
    ClearScreen,
    Suspend,
    Resume,

    CycleFocus,
    CycleFocusBackwards,

    OpenHelp,
    CloseModal,
    OpenEmulators,

    DeviceListUp,
    DeviceListDown,
    DisconnectDevice,
    RefreshDevices,
    DevicesUpdated(Vec<Device>),

    EmulatorListUp,
    EmulatorListDown,
    KillEmulator,
    EmulatorSelect,
}
