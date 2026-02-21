use serde::{Deserialize, Serialize};

pub mod content;
pub mod devices;
pub mod emulators;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pane {
    #[default]
    DeviceList,
    Emulators,
    Content,
}

impl Pane {
    pub fn next(&self) -> Self {
        match self {
            Self::DeviceList => Self::Emulators,
            Self::Emulators => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }
}
