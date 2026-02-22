pub mod content;
pub mod devices;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Pane {
    #[default]
    DeviceList = 0,
    Content = 1,
}

const PANE_COUNT: u8 = 2;

impl Pane {
    fn from_index(i: u8) -> Self {
        match i {
            0 => Pane::DeviceList,
            1 => Pane::Content,
            _ => unreachable!(),
        }
    }

    pub fn next(self) -> Self {
        Self::from_index((self as u8 + 1) % PANE_COUNT)
    }

    pub fn prev(self) -> Self {
        Self::from_index((self as u8 + PANE_COUNT - 1) % PANE_COUNT)
    }
}
