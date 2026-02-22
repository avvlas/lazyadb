use crossterm::event::KeyEvent;

use crate::adb::device::Device;
use crate::adb::device_info::DeviceInfo;

#[derive(Debug)]
pub enum Msg {
    Tick,
    DevicesUpdated(Vec<Device>),
    DeviceSelected(Option<Device>),
    DeviceInfoUpdated(DeviceInfo),
    KeyPress(KeyEvent),
}
