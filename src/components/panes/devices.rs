use std::str::FromStr;
use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::adb::device::{ConnectionType, Device, DeviceState};
use crate::command::Command;
use crate::components::{Component, DrawContext, panes::Pane};
use crate::config::keymap::SectionKeymap;
use crate::msg::Msg;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq)]
enum DeviceAction {
    Up,
    Down,
    Disconnect,
    Refresh,
    OpenEmulators,
}

impl FromStr for DeviceAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Up" => Ok(Self::Up),
            "Down" => Ok(Self::Down),
            "Disconnect" => Ok(Self::Disconnect),
            "Refresh" => Ok(Self::Refresh),
            "OpenEmulators" => Ok(Self::OpenEmulators),
            _ => Err(()),
        }
    }
}

pub struct DevicesPane {
    devices: Vec<Device>,
    selected_index: usize,
    last_refresh: Instant,
    keymap: SectionKeymap,
    last_selected_serial: Option<String>,
}

impl DevicesPane {
    pub fn new(devices: Vec<Device>, keymap: SectionKeymap) -> Self {
        Self {
            devices,
            selected_index: 0,
            last_refresh: Instant::now(),
            keymap,
            last_selected_serial: None,
        }
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices
    }

    pub fn selected_device(&self) -> Option<&Device> {
        self.devices.get(self.selected_index)
    }

    fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.devices.len() - 1);
        }
    }

    fn clamp_selection(&mut self) {
        if self.devices.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.devices.len() - 1);
        }
    }

    fn selection_changed_command(&mut self) -> Option<Command> {
        let new_serial = self.selected_device().map(|d| d.serial.clone());
        if new_serial != self.last_selected_serial {
            self.last_selected_serial = new_serial;
            Some(Command::DeviceSelected(self.selected_device().cloned()))
        } else {
            None
        }
    }

    fn disconnect_command(&self) -> Option<Command> {
        let device = self.selected_device()?;
        match device.connection_type {
            ConnectionType::Emulator => Some(Command::KillEmulator(device.serial.clone())),
            ConnectionType::Tcp => Some(Command::DisconnectDevice(device.serial.clone())),
            ConnectionType::Usb => None,
        }
    }

    fn handle_action(&mut self, action: DeviceAction) -> Vec<Command> {
        match action {
            DeviceAction::Up => {
                self.select_previous();
                if let Some(cmd) = self.selection_changed_command() {
                    return vec![cmd];
                }
            }
            DeviceAction::Down => {
                self.select_next();
                if let Some(cmd) = self.selection_changed_command() {
                    return vec![cmd];
                }
            }
            DeviceAction::Disconnect => {
                if let Some(cmd) = self.disconnect_command() {
                    return vec![cmd];
                }
            }
            DeviceAction::Refresh => {
                return vec![Command::RefreshDevices];
            }
            DeviceAction::OpenEmulators => {
                return vec![Command::OpenEmulatorsModal];
            }
        }
        return Vec::new();
    }
}

impl Component for DevicesPane {
    fn update(&mut self, msg: &Msg) -> Vec<Command> {
        match msg {
            Msg::KeyPress(key) => {
                let key_seq = vec![*key];
                let action = self
                    .keymap
                    .get(&key_seq)
                    .and_then(|s| DeviceAction::from_str(s).ok());

                match action {
                    Some(action) => self.handle_action(action),
                    None => Vec::new(),
                }
            }
            Msg::Tick => {
                if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
                    self.last_refresh = Instant::now();
                    vec![Command::RefreshDevices]
                } else {
                    Vec::new()
                }
            }
            Msg::DevicesUpdated(devices) => {
                self.devices = devices.clone();
                self.clamp_selection();
                match self.selection_changed_command() {
                    Some(cmd) => vec![cmd],
                    None => Vec::new(),
                }
            }
            _ => Vec::new(),
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        let focused = ctx.focus == Pane::DeviceList;
        let border_color = if focused {
            Color::Green
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" DEVICES ")
            .border_style(Style::default().fg(border_color));

        if self.devices.is_empty() {
            let paragraph = Paragraph::new("(no devices)").block(block);
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .devices
            .iter()
            .map(|device| {
                let (icon, icon_color) = match device.state {
                    DeviceState::Online => ("●", Color::Green),
                    DeviceState::Offline => ("○", Color::Red),
                    DeviceState::Unauthorized => ("⚠", Color::Yellow),
                    DeviceState::Unknown(_) => ("?", Color::DarkGray),
                };

                let conn_tag = match device.connection_type {
                    ConnectionType::Usb => " [USB]",
                    ConnectionType::Tcp => " [TCP]",
                    ConnectionType::Emulator => " [EMU]",
                };

                let name = device.display_name();

                let line = Line::from(vec![
                    Span::styled(icon.to_string(), Style::default().fg(icon_color)),
                    Span::raw(format!(" {}", name)),
                    Span::styled(conn_tag, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut list_state = ListState::default().with_selected(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut list_state);
    }

    fn id(&self) -> &'static str {
        "DeviceList"
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    fn make_device(serial: &str, conn: ConnectionType) -> Device {
        Device {
            serial: serial.into(),
            state: DeviceState::Online,
            model: None,
            product: None,
            transport_id: None,
            connection_type: conn,
        }
    }

    fn make_keymap() -> SectionKeymap {
        let mut keymap = SectionKeymap::new();
        let j = vec![KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty())];
        let k = vec![KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty())];
        let d = vec![KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty())];
        let r = vec![KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty())];
        let e = vec![KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty())];
        keymap.insert(j, "Down".into());
        keymap.insert(k, "Up".into());
        keymap.insert(d, "Disconnect".into());
        keymap.insert(r, "Refresh".into());
        keymap.insert(e, "OpenEmulators".into());
        keymap
    }

    fn key(c: char) -> Msg {
        Msg::KeyPress(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
    }

    fn pane_with_devices(count: usize) -> DevicesPane {
        let devices: Vec<Device> = (0..count)
            .map(|i| make_device(&format!("DEV{i}"), ConnectionType::Usb))
            .collect();
        DevicesPane::new(devices, make_keymap())
    }

    #[test]
    fn device_action_from_str() {
        assert_eq!(DeviceAction::from_str("Up"), Ok(DeviceAction::Up));
        assert_eq!(DeviceAction::from_str("Down"), Ok(DeviceAction::Down));
        assert_eq!(
            DeviceAction::from_str("Disconnect"),
            Ok(DeviceAction::Disconnect)
        );
        assert_eq!(DeviceAction::from_str("Refresh"), Ok(DeviceAction::Refresh));
        assert_eq!(
            DeviceAction::from_str("OpenEmulators"),
            Ok(DeviceAction::OpenEmulators)
        );
        assert!(DeviceAction::from_str("invalid").is_err());
    }

    #[test]
    fn new_pane_starts_at_index_zero() {
        let pane = pane_with_devices(3);
        assert_eq!(pane.selected_index, 0);
        assert_eq!(pane.selected_device().unwrap().serial, "DEV0");
    }

    #[test]
    fn select_next_moves_down() {
        let mut pane = pane_with_devices(3);
        pane.select_next();
        assert_eq!(pane.selected_index, 1);
        pane.select_next();
        assert_eq!(pane.selected_index, 2);
    }

    #[test]
    fn select_next_clamps_at_end() {
        let mut pane = pane_with_devices(2);
        pane.select_next();
        pane.select_next();
        pane.select_next();
        assert_eq!(pane.selected_index, 1);
    }

    #[test]
    fn select_next_on_empty_does_nothing() {
        let mut pane = pane_with_devices(0);
        pane.select_next();
        assert_eq!(pane.selected_index, 0);
    }

    #[test]
    fn select_previous_moves_up() {
        let mut pane = pane_with_devices(3);
        pane.selected_index = 2;
        pane.select_previous();
        assert_eq!(pane.selected_index, 1);
    }

    #[test]
    fn select_previous_clamps_at_zero() {
        let mut pane = pane_with_devices(3);
        pane.select_previous();
        assert_eq!(pane.selected_index, 0);
    }

    #[test]
    fn keypress_down_moves_selection() {
        let mut pane = pane_with_devices(3);
        let cmds = pane.update(&key('j'));
        assert_eq!(pane.selected_index, 1);
        assert!(cmds.iter().any(|c| matches!(c, Command::DeviceSelected(_))));
    }

    #[test]
    fn keypress_up_moves_selection() {
        let mut pane = pane_with_devices(3);
        pane.selected_index = 2;
        pane.last_selected_serial = Some("DEV2".into());
        let cmds = pane.update(&key('k'));
        assert_eq!(pane.selected_index, 1);
        assert!(cmds.iter().any(|c| matches!(c, Command::DeviceSelected(_))));
    }

    #[test]
    fn unknown_key_returns_no_commands() {
        let mut pane = pane_with_devices(3);
        let cmds = pane.update(&key('z'));
        assert!(cmds.is_empty());
    }

    #[test]
    fn refresh_key_returns_refresh_command() {
        let mut pane = pane_with_devices(1);
        let cmds = pane.update(&key('r'));
        assert!(cmds.iter().any(|c| matches!(c, Command::RefreshDevices)));
    }

    #[test]
    fn open_emulators_key_returns_command() {
        let mut pane = pane_with_devices(1);
        let cmds = pane.update(&key('e'));
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Command::OpenEmulatorsModal))
        );
    }

    #[test]
    fn disconnect_emulator_returns_kill_command() {
        let devices = vec![make_device("emulator-5554", ConnectionType::Emulator)];
        let mut pane = DevicesPane::new(devices, make_keymap());
        let cmds = pane.update(&key('d'));
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Command::KillEmulator(s) if s == "emulator-5554"))
        );
    }

    #[test]
    fn disconnect_tcp_returns_disconnect_command() {
        let devices = vec![make_device("192.168.1.1:5555", ConnectionType::Tcp)];
        let mut pane = DevicesPane::new(devices, make_keymap());
        let cmds = pane.update(&key('d'));
        assert!(
            cmds.iter()
                .any(|c| matches!(c, Command::DisconnectDevice(s) if s == "192.168.1.1:5555"))
        );
    }

    #[test]
    fn disconnect_usb_returns_no_command() {
        let devices = vec![make_device("USB001", ConnectionType::Usb)];
        let mut pane = DevicesPane::new(devices, make_keymap());
        let cmds = pane.update(&key('d'));
        assert!(cmds.is_empty());
    }

    #[test]
    fn devices_updated_replaces_list() {
        let mut pane = pane_with_devices(2);
        pane.selected_index = 1;
        pane.last_selected_serial = Some("DEV1".into());

        let new_devices = vec![
            make_device("NEW0", ConnectionType::Usb),
            make_device("NEW1", ConnectionType::Usb),
            make_device("NEW2", ConnectionType::Usb),
        ];
        pane.update(&Msg::DevicesUpdated(new_devices));

        assert_eq!(pane.devices().len(), 3);
        assert_eq!(pane.selected_index, 1);
    }

    #[test]
    fn devices_updated_clamps_selection() {
        let mut pane = pane_with_devices(5);
        pane.selected_index = 4;
        pane.last_selected_serial = Some("DEV4".into());

        let new_devices = vec![make_device("A", ConnectionType::Usb)];
        pane.update(&Msg::DevicesUpdated(new_devices));

        assert_eq!(pane.selected_index, 0);
    }

    #[test]
    fn devices_updated_to_empty_resets_selection() {
        let mut pane = pane_with_devices(3);
        pane.selected_index = 2;
        pane.last_selected_serial = Some("DEV2".into());

        pane.update(&Msg::DevicesUpdated(vec![]));

        assert_eq!(pane.selected_index, 0);
        assert!(pane.devices().is_empty());
    }

    #[test]
    fn tick_before_interval_returns_no_commands() {
        let mut pane = pane_with_devices(1);
        pane.last_refresh = Instant::now();
        let cmds = pane.update(&Msg::Tick);
        assert!(cmds.is_empty());
    }

    #[test]
    fn tick_after_interval_returns_refresh() {
        let mut pane = pane_with_devices(1);
        pane.last_refresh = Instant::now() - REFRESH_INTERVAL - Duration::from_millis(100);
        let cmds = pane.update(&Msg::Tick);
        assert!(cmds.iter().any(|c| matches!(c, Command::RefreshDevices)));
    }

    #[test]
    fn selection_emits_only_on_change() {
        let mut pane = pane_with_devices(3);
        // First move emits
        let cmds = pane.update(&key('j'));
        assert!(!cmds.is_empty());

        // Same position, no emission
        let prev_serial = pane.last_selected_serial.clone();
        let cmds = pane.update(&key('k'));
        // Moved back to 0, serial changed
        assert_ne!(pane.last_selected_serial, prev_serial);
        assert!(!cmds.is_empty());
    }

    #[test]
    fn devices_accessor_returns_slice() {
        let pane = pane_with_devices(3);
        assert_eq!(pane.devices().len(), 3);
        assert_eq!(pane.devices()[0].serial, "DEV0");
    }
}
