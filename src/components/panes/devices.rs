use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::adb::device::{ConnectionType, Device, DeviceState};
use crate::command::Command;
use crate::components::{DrawContext, panes::Pane};
use crate::config::keymap::SectionKeymap;
use crate::{components::Component, msg::Msg};

enum DeviceAction {
    Up,
    Down,
    Disconnect,
    Refresh,
    OpenEmulators,
}

impl DeviceAction {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            "Disconnect" => Some(Self::Disconnect),
            "Refresh" => Some(Self::Refresh),
            "OpenEmulators" => Some(Self::OpenEmulators),
            _ => None,
        }
    }
}

pub struct DevicesPane {
    pub items: Vec<Device>,
    selected_index: usize,
    last_refresh: std::time::Instant,
    keymap: SectionKeymap,
}

impl DevicesPane {
    pub fn new(items: Vec<Device>, keymap: SectionKeymap) -> Self {
        Self {
            items,
            selected_index: 0,
            last_refresh: std::time::Instant::now(),
            keymap,
        }
    }
}

const DEVICES_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

impl Component for DevicesPane {
    fn update(&mut self, action: &Msg) -> Vec<Command> {
        match action {
            Msg::KeyPress(key) => {
                let key_seq = vec![*key];
                let Some(action_str) = self.keymap.get(&key_seq) else {
                    return Vec::new();
                };
                let Some(action) = DeviceAction::from_str(action_str) else {
                    return Vec::new();
                };

                match action {
                    DeviceAction::Up => {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }
                    DeviceAction::Down => {
                        let count = self.items.len();
                        if count > 0 {
                            self.selected_index = (self.selected_index + 1).min(count - 1);
                        }
                    }
                    DeviceAction::Disconnect => {
                        if let Some(device) = self.items.get(self.selected_index) {
                            match device.connection_type {
                                ConnectionType::Emulator => {
                                    return vec![Command::KillEmulator(device.serial.clone())];
                                }
                                ConnectionType::Tcp => {
                                    return vec![Command::DisconnectDevice(device.serial.clone())];
                                }
                                ConnectionType::Usb => {}
                            }
                        }
                    }
                    DeviceAction::Refresh => {
                        return vec![Command::RefreshDevices];
                    }
                    DeviceAction::OpenEmulators => {
                        return vec![Command::OpenEmulatorsModal];
                    }
                }
            }
            Msg::Tick => {
                if self.last_refresh.elapsed() >= DEVICES_REFRESH_INTERVAL {
                    self.last_refresh = std::time::Instant::now();
                    return vec![Command::RefreshDevices];
                }
            }
            Msg::DevicesUpdated(devices) => {
                self.items = devices.clone();
                let count = self.items.len();
                if count > 0 {
                    self.selected_index = self.selected_index.min(count - 1);
                } else {
                    self.selected_index = 0;
                }
            }
        }
        Vec::new()
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

        if self.items.is_empty() {
            let paragraph = Paragraph::new("(no devices)").block(block);
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .items
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
