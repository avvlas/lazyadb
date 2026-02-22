use crate::adb::device::Device;

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Tick,
    DevicesUpdated(Vec<Device>),
}
