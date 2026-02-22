use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    adb::emulator::Avd,
    command::Command,
    components::{Component, DrawContext, modals::centered_rect, panes::Pane},
    config::keymap::SectionKeymap,
    msg::Msg,
};

pub struct EmulatorsModal {
    items: Vec<Avd>,
    selected_index: usize,
    keymap: SectionKeymap,
}

impl EmulatorsModal {
    pub fn new(items: Vec<Avd>, keymap: SectionKeymap) -> Self {
        Self {
            items,
            selected_index: 0,
            keymap,
        }
    }
}

enum EmulatorAction {
    Up,
    Down,
    Select,
    Kill,
}

impl EmulatorAction {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            "Select" => Some(Self::Select),
            "Kill" => Some(Self::Kill),
            _ => None,
        }
    }
}

impl Component for EmulatorsModal {
    fn handle_key(&mut self, key: KeyEvent) -> Vec<Command> {
        let key_seq = vec![key];
        let Some(action_str) = self.keymap.get(&key_seq) else {
            return Vec::new();
        };
        let Some(action) = EmulatorAction::from_str(action_str) else {
            return Vec::new();
        };

        match action {
            EmulatorAction::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            EmulatorAction::Down => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
                }
            }
            EmulatorAction::Kill => {
                if let Some(avd) = self.items.get(self.selected_index)
                    && let Some(serial) = &avd.running_serial
                {
                    return vec![Command::KillEmulator(serial.clone())];
                }
            }
            EmulatorAction::Select => {
                if let Some(avd) = self.items.get(self.selected_index) {
                    if avd.is_running() {
                        return vec![Command::CloseEmulatorsModal, Command::Focus(Pane::Content)];
                    } else {
                        return vec![Command::StartEmulator(avd.name.clone())];
                    }
                }
            }
        }
        Vec::new()
    }

    fn update(&mut self, _action: &Msg) -> Vec<Command> {
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
