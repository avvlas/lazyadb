use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    command::Command,
    components::{Component, DrawContext, modals::centered_rect},
    msg::Msg,
};

pub struct HelpModal;

impl HelpModal {
    pub fn new() -> Self {
        Self
    }
}

impl Component for HelpModal {
    fn update(&mut self, _action: &Msg) -> Vec<Command> {
        Vec::new()
    }

    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &DrawContext) {
        let rect = centered_rect(50, 50, area);
        frame.render_widget(Clear, rect);

        let help_text = "\
Keybindings
───────────
q         Quit
Tab       Cycle focus (Devices → Content)
j / ↓     Select next item
k / ↑     Select previous item
x         Disconnect device (TCP/Emulator)
e         Open emulators popup
Enter     Start / select emulator (popup)
x         Kill running emulator (popup)
?         Toggle help
Esc       Close modal";

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" HELP ")
            .border_style(Style::default().fg(Color::Green));
        let paragraph = Paragraph::new(help_text).block(block);
        frame.render_widget(paragraph, rect);
    }

    fn id(&self) -> &'static str {
        "Help"
    }
}
