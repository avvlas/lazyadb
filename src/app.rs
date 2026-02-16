use std::time::Instant;

use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::adb::client::AdbClient;
use crate::adb::device::Device;
use crate::event::{self, AppEvent};
use crate::keys;
use crate::ui;

pub enum FocusPanel {
    DeviceList,
    Content,
}

impl FocusPanel {
    pub fn next(&self) -> Self {
        match self {
            Self::DeviceList => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::DeviceList => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }
}

pub struct App {
    pub running: bool,
    pub focus: FocusPanel,
    pub show_help: bool,
    pub adb: AdbClient,
    pub devices: Vec<Device>,
    pub selected_device_index: usize,
    pub active_device_serial: Option<String>,
    last_refresh: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();

        Ok(Self {
            running: true,
            focus: FocusPanel::DeviceList,
            show_help: false,
            adb,
            devices,
            selected_device_index: 0,
            active_device_serial: None,
            last_refresh: Instant::now(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| ui::draw(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::poll_event()? {
            AppEvent::Key(key) => keys::handle_key_event(self, key),
            AppEvent::Tick => {
                if self.last_refresh.elapsed() >= std::time::Duration::from_secs(2) {
                    self.refresh_devices();
                }
            }
        }
        Ok(())
    }

    pub fn refresh_devices(&mut self) {
        if let Ok(devices) = self.adb.devices() {
            self.devices = devices;
        }
        // Clamp selection index
        if !self.devices.is_empty() {
            self.selected_device_index = self.selected_device_index.min(self.devices.len() - 1);
        } else {
            self.selected_device_index = 0;
        }
        // Clear active device if it disappeared
        if let Some(serial) = &self.active_device_serial {
            if !self.devices.iter().any(|d| &d.serial == serial) {
                self.active_device_serial = None;
            }
        }
        self.last_refresh = Instant::now();
    }

    pub fn select_next_device(&mut self) {
        if !self.devices.is_empty() {
            self.selected_device_index =
                (self.selected_device_index + 1).min(self.devices.len() - 1);
        }
    }

    pub fn select_prev_device(&mut self) {
        self.selected_device_index = self.selected_device_index.saturating_sub(1);
    }

    pub fn confirm_device_selection(&mut self) {
        if let Some(device) = self.devices.get(self.selected_device_index) {
            self.active_device_serial = Some(device.serial.clone());
        }
    }

    pub fn active_device(&self) -> Option<&Device> {
        let serial = self.active_device_serial.as_ref()?;
        self.devices.iter().find(|d| &d.serial == serial)
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn cycle_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn cycle_focus_backwards(&mut self) {
        self.focus = self.focus.previous();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn close_modal(&mut self) {
        self.show_help = false;
    }
}
