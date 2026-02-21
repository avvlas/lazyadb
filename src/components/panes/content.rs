use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{action::Action, components::Component};
use crate::{
    command::Command,
    components::{DrawContext, panes::Pane},
};

pub struct ContentPane;

impl ContentPane {
    pub fn new() -> Self {
        Self
    }
}

impl Component for ContentPane {
    fn update(&mut self, _action: &Action) -> Vec<Command> {
        Vec::new()
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext) {
        let focused = ctx.focus == Pane::Content;
        let border_color = if focused {
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
    }

    fn id(&self) -> &'static str {
        "Content"
    }
}
