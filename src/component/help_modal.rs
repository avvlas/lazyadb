use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{action::Action, component::Component};

pub struct HelpModalComponent {
    visible: bool,
}

impl HelpModalComponent {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Component for HelpModalComponent {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::ToggleHelp => {
                self.visible = !self.visible;
            }
            Action::CloseModal => {
                self.visible = false;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        if !self.visible {
            return Ok(());
        }

        let rect = centered_rect(50, 50, area);
        frame.render_widget(Clear, rect);

        let help_text = "\
Keybindings
───────────
q         Quit
Tab       Cycle focus (Devices → Emulators → Content)
j / ↓     Select next item
k / ↑     Select previous item
x         Kill running emulator (Emulators panel)
?         Toggle help
Esc       Close modal";

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" HELP ")
            .border_style(Style::default().fg(Color::Green));
        let paragraph = Paragraph::new(help_text).block(block);
        frame.render_widget(paragraph, rect);
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
