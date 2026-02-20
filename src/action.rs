use serde::{Deserialize, Serialize};
use strum::Display;

use crate::adb::device::Device;
use crate::adb::emulator::Avd;
use crate::app::FocusPanel;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Quit,
    ClearScreen,
    Suspend,
    Resume,
    Error(String),
    Help,

    CycleFocus,

    ToggleHelp,
    CloseModal,

    DeviceListUp,
    DeviceListDown,
    RefreshDevices,

    EmulatorListUp,
    EmulatorListDown,
    KillEmulator,
    EmulatorSelect,

    #[serde(skip)]
    DevicesUpdated(Vec<Device>),
    #[serde(skip)]
    EmulatorsUpdated(Vec<Avd>),
    #[serde(skip)]
    FocusChanged(FocusPanel),
}
