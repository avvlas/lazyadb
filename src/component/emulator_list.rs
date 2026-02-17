use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::action::Action;
use crate::adb::emulator::Avd;
use crate::app::FocusPanel;
use crate::component::Component;

pub struct EmulatorListComponent {
    emulators: Vec<Avd>,
    selected_index: usize,
    focused: bool,
}

impl EmulatorListComponent {
    pub fn new(emulators: Vec<Avd>) -> Self {
        Self {
            emulators,
            selected_index: 0,
            focused: false,
        }
    }
}

impl Component for EmulatorListComponent {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::EmulatorsUpdated(emulators) => {
                self.emulators = emulators;
                if !self.emulators.is_empty() {
                    self.selected_index = self.selected_index.min(self.emulators.len() - 1);
                } else {
                    self.selected_index = 0;
                }
            }
            Action::EmulatorListUp => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            Action::EmulatorListDown => {
                if !self.emulators.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.emulators.len() - 1);
                }
            }
            Action::FocusChanged(panel) => {
                self.focused = panel == FocusPanel::Emulators;
            }
            Action::KillEmulator => {
                if let Some(avd) = self.emulators.get(self.selected_index)
                    && let Some(serial) = &avd.running_serial
                {
                    return Ok(Some(Action::KillEmulatorBySerial(serial.clone())));
                }
            }
            Action::EmulatorSelect => {
                if let Some(avd) = self.emulators.get(self.selected_index) {
                    if avd.is_running() {
                        return Ok(Some(Action::FocusChanged(FocusPanel::Content)));
                    } else {
                        return Ok(Some(Action::StartEmulatorByName(avd.name.clone())));
                    }
                }
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
            .title(" EMULATORS ")
            .border_style(Style::default().fg(border_color));

        if self.emulators.is_empty() {
            let paragraph = Paragraph::new("(no AVDs)").block(block);
            frame.render_widget(paragraph, area);
            return Ok(());
        }

        let items: Vec<ListItem> = self
            .emulators
            .iter()
            .map(|avd| {
                let (icon, icon_color) = if avd.is_running() {
                    ("▶", Color::Green)
                } else {
                    ("■", Color::DarkGray)
                };

                let suffix = if avd.is_running() { " (running)" } else { "" };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                    Span::raw(format!("{}{}", avd.display_name(), suffix)),
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
