use strum::Display;

use crate::adb::device::Device;
use crate::adb::emulator::Avd;
use crate::app::FocusPanel;

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
    Focus(FocusPanel),

    ToggleHelp,
    CloseModal,

    DeviceListUp,
    DeviceListDown,
    RefreshDevices,
    DevicesUpdated(Vec<Device>),

    EmulatorListUp,
    EmulatorListDown,
    KillEmulator,
    EmulatorSelect,
    EmulatorsUpdated(Vec<Avd>),
}
