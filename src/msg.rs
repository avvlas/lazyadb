use crossterm::event::KeyEvent;

use crate::adb::device::Device;

#[derive(Debug, PartialEq)]
pub enum Msg {
    Tick,
    DevicesUpdated(Vec<Device>),
    KeyPress(KeyEvent),
}
