use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::Command;
use crate::config::Config;
use crate::msg::Msg;
use panes::Pane;

pub mod modals;
pub mod panes;

#[allow(dead_code)]
pub struct DrawContext<'a> {
    pub focus: Pane,
    pub config: &'a Config,
}

pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Vec<Command> {
        let _ = key;
        Vec::new()
    }
    fn update(&mut self, action: &Msg) -> Vec<Command>;
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext);
    fn id(&self) -> &str;
}
