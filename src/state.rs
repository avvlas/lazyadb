use std::time::Instant;

use crate::adb::client::AdbClient;
use crate::adb::device::Device;
use crate::adb::emulator::Avd;
use crate::config::Config;
use crate::panes::Pane;

#[allow(dead_code)]
pub struct State {
    pub running: bool,
    pub should_suspend: bool,
    pub focus: Pane,
    pub config: Config,
    pub adb: AdbClient,
    pub last_refresh: Instant,
    pub devices: DevicesState,
    pub emulators: EmulatorsState,
    pub content: ContentState,
    pub modal: ModalState,
}

pub struct DevicesState {
    pub items: Vec<Device>,
    pub selected_index: usize,
}

pub struct EmulatorsState {
    pub items: Vec<Avd>,
    pub selected_index: usize,
}

pub struct ContentState {}

pub enum ModalState {
    Help,
    Emulators,
    None,
}
