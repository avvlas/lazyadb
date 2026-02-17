use std::time::Instant;

use color_eyre::Result;
use ratatui::DefaultTerminal;
use serde::{Deserialize, Serialize};

use crate::adb::client::AdbClient;
use crate::adb::device::{ConnectionType, Device};
use crate::adb::emulator::Avd;
use crate::component;
use crate::config::Config;
use crate::event::{self, AppEvent};
use crate::keys;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FocusPanel {
    #[default]
    DeviceList,
    Emulators,
    Content,
}

impl FocusPanel {
    pub fn next(&self) -> Self {
        match self {
            Self::DeviceList => Self::Emulators,
            Self::Emulators => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::DeviceList => Self::Content,
            Self::Emulators => Self::DeviceList,
            Self::Content => Self::Emulators,
        }
    }
}

pub struct App {
    pub running: bool,
    pub focus: FocusPanel,
    pub show_help: bool,
    pub config: Config,
    pub adb: AdbClient,
    pub devices: Vec<Device>,
    pub selected_device_index: usize,
    pub emulators: Vec<Avd>,
    pub selected_emulator_index: usize,
    last_refresh: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();

        let emulators = adb.avds_with_status(&devices);

        Ok(Self {
            running: true,
            focus: FocusPanel::DeviceList,
            show_help: false,
            config,
            adb,
            devices,
            selected_device_index: 0,
            emulators,
            selected_emulator_index: 0,
            last_refresh: Instant::now(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| component::draw(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::poll_event()? {
            AppEvent::Key(key) => keys::handle_key_event(self, key, &self.config.clone()),
            AppEvent::Tick => {
                if self.last_refresh.elapsed() >= std::time::Duration::from_secs(2) {
                    self.refresh_devices();
                }
            }
        }
        Ok(())
    }

    pub fn physical_devices(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|d| d.connection_type != ConnectionType::Emulator)
            .collect()
    }

    pub fn refresh_devices(&mut self) {
        if let Ok(devices) = self.adb.devices() {
            self.devices = devices;
        }
        // Clamp device selection against physical devices
        let physical_count = self.physical_devices().len();
        if physical_count > 0 {
            self.selected_device_index = self.selected_device_index.min(physical_count - 1);
        } else {
            self.selected_device_index = 0;
        }
        // Refresh emulator list
        self.emulators = self.adb.avds_with_status(&self.devices);
        if !self.emulators.is_empty() {
            self.selected_emulator_index =
                self.selected_emulator_index.min(self.emulators.len() - 1);
        } else {
            self.selected_emulator_index = 0;
        }
        self.last_refresh = Instant::now();
    }

    pub fn select_next_device(&mut self) {
        let count = self.physical_devices().len();
        if count > 0 {
            self.selected_device_index = (self.selected_device_index + 1).min(count - 1);
        }
    }

    pub fn select_prev_device(&mut self) {
        self.selected_device_index = self.selected_device_index.saturating_sub(1);
    }

    pub fn select_next_emulator(&mut self) {
        if !self.emulators.is_empty() {
            self.selected_emulator_index =
                (self.selected_emulator_index + 1).min(self.emulators.len() - 1);
        }
    }

    pub fn select_prev_emulator(&mut self) {
        self.selected_emulator_index = self.selected_emulator_index.saturating_sub(1);
    }

    pub fn kill_selected_emulator(&mut self) {
        if let Some(avd) = self.emulators.get(self.selected_emulator_index)
            && let Some(serial) = &avd.running_serial
        {
            let _ = self.adb.kill_emulator(serial);
        }
    }

    pub fn select_emulator(&mut self) {
        if let Some(avd) = self.emulators.get(self.selected_emulator_index) {
            if avd.is_running() {
                self.focus = FocusPanel::Content;
            } else {
                let _ = self.adb.start_emulator(&avd.name);
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn cycle_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn close_modal(&mut self) {
        self.show_help = false;
    }
}
