use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    action::Action,
    adb::emulator::Avd,
    command::Command,
    components::modals::Modal,
    components::{Component, DrawContext, modals::centered_rect, panes::Pane},
};

pub struct EmulatorsModal {
    pub items: Vec<Avd>,
    pub selected_index: usize,
}

impl EmulatorsModal {
    pub fn new(items: Vec<Avd>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }
}

impl Component for EmulatorsModal {
    fn update(&mut self, action: &Action) -> Vec<Command> {
        match action {
            Action::EmulatorListUp => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            Action::EmulatorListDown => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
                }
            }
            Action::KillEmulator => {
                if let Some(avd) = self.items.get(self.selected_index)
                    && let Some(serial) = &avd.running_serial
                {
                    return vec![Command::KillEmulator(serial.clone())];
                }
            }
            Action::EmulatorSelect => {
                if let Some(avd) = self.items.get(self.selected_index) {
                    if avd.is_running() {
                        return vec![Command::CloseEmulatorsModal, Command::Focus(Pane::Content)];
                    } else {
                        return vec![Command::StartEmulator(avd.name.clone())];
                    }
                }
            }
            _ => {}
        }
        Vec::new()
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        let rect = centered_rect(50, 50, area);
        frame.render_widget(Clear, rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" EMULATORS ")
            .border_style(Style::default().fg(Color::Green));

        if self.items.is_empty() {
            let paragraph = Paragraph::new("(no AVDs)").block(block);
            frame.render_widget(paragraph, rect);
            return;
        }

        let items: Vec<ListItem> = self
            .items
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

        let mut list_state = ListState::default().with_selected(Some(self.selected_index));
        frame.render_stateful_widget(list, rect, &mut list_state);
    }

    fn id(&self) -> &'static str {
        "Emulators"
    }
}
