use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Quit,
    Suspend,
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
}
