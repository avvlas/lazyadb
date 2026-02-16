use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::event::{self, AppEvent};
use crate::keys;
use crate::ui;

pub enum FocusPanel {
    DeviceList,
    Content,
}

impl FocusPanel {
    pub fn next(&self) -> Self {
        match self {
            Self::DeviceList => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::DeviceList => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }
}

pub struct App {
    pub running: bool,
    pub focus: FocusPanel,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            focus: FocusPanel::DeviceList,
            show_help: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| ui::draw(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::poll_event()? {
            AppEvent::Key(key) => keys::handle_key_event(self, key),
            AppEvent::Tick => {}
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn cycle_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn cycle_focus_backwards(&mut self) {
        self.focus = self.focus.previous();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn close_modal(&mut self) {
        self.show_help = false;
    }
}
