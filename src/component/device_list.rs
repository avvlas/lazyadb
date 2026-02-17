use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::action::Action;
use crate::adb::device::{ConnectionType, Device, DeviceState};
use crate::app::FocusPanel;
use crate::component::Component;

pub struct DeviceListComponent {
    devices: Vec<Device>,
    selected_index: usize,
    focused: bool,
}

impl DeviceListComponent {
    pub fn new(devices: Vec<Device>) -> Self {
        Self {
            devices,
            selected_index: 0,
            focused: true,
        }
    }

    fn physical_devices(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|d| d.connection_type != ConnectionType::Emulator)
            .collect()
    }
}

impl Component for DeviceListComponent {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::DevicesUpdated(devices) => {
                self.devices = devices;
                let count = self.physical_devices().len();
                if count > 0 {
                    self.selected_index = self.selected_index.min(count - 1);
                } else {
                    self.selected_index = 0;
                }
            }
            Action::DeviceListUp => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            Action::DeviceListDown => {
                let count = self.physical_devices().len();
                if count > 0 {
                    self.selected_index = (self.selected_index + 1).min(count - 1);
                }
            }
            Action::FocusChanged(panel) => {
                self.focused = panel == FocusPanel::DeviceList;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let border_color = if self.focused {
            Color::Green
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" DEVICES ")
            .border_style(Style::default().fg(border_color));

        let physical = self.physical_devices();

        if physical.is_empty() {
            let paragraph = Paragraph::new("(no devices)").block(block);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let items: Vec<ListItem> = physical
            .iter()
            .map(|device| {
                let (icon, icon_color) = match device.state {
                    DeviceState::Online => ("●", Color::Green),
                    DeviceState::Offline => ("○", Color::Red),
                    DeviceState::Unauthorized => ("⚠", Color::Yellow),
                    DeviceState::Unknown(_) => ("?", Color::DarkGray),
                };

                let name = device.display_name();

                let line = Line::from(vec![
                    Span::styled(icon.to_string(), Style::default().fg(icon_color)),
                    Span::raw(name.to_string()),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut state = ListState::default().with_selected(Some(self.selected_index));
        frame.render_stateful_widget(list, area, &mut state);
        Ok(())
    }
}
