use std::time::Instant;

use tokio::sync::mpsc;

use crate::message::Msg;
use crate::adb::client::AdbClient;
use crate::adb::device::Device;
use crate::adb::emulator::Avd;
use crate::app::FocusPanel;
use crate::config::Config;

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
    None,
}

#[allow(dead_code)]
pub struct State {
    pub running: bool,
    pub should_suspend: bool,
    pub focus: FocusPanel,
    pub config: Config,
    pub adb: AdbClient,
    pub last_refresh: Instant,
    pub action_tx: mpsc::UnboundedSender<Msg>,
    pub action_rx: mpsc::UnboundedReceiver<Msg>,
    pub devices: DevicesState,
    pub emulators: EmulatorsState,
    pub content: ContentState,
    pub modal: ModalState,
}
