use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::action::Action;
use crate::app::FocusPanel;
use crate::component::Component;

pub struct ContentAreaComponent {
    focused: bool,
}

impl ContentAreaComponent {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for ContentAreaComponent {
    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        if let Action::FocusChanged(panel) = action {
            self.focused = panel == FocusPanel::Content;
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
            .title(" CONTENT ")
            .border_style(Style::default().fg(border_color));
        let paragraph = Paragraph::new("Select a device to begin").block(block);
        frame.render_widget(paragraph, area);
        Ok(())
    }
}
