use ratatui::Frame;
use ratatui::layout::Rect;

use crate::action::Action;
use crate::command::Command;
use crate::config::Config;
use panes::Pane;

pub mod modals;
pub mod panes;

#[allow(dead_code)]
pub struct DrawContext<'a> {
    pub focus: Pane,
    pub config: &'a Config,
}

pub trait Component {
    fn update(&mut self, action: &Action) -> Vec<Command>;
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &DrawContext);
    fn id(&self) -> &str;
}
