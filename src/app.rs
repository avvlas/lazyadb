use std::{collections::HashMap, time::Instant};

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use tokio::sync::mpsc;
use tracing::debug;

use crate::command::Command;
use crate::config::Config;
use crate::message::Action;
use crate::modals;
use crate::panes;
use crate::state::{ContentState, DevicesState, EmulatorsState, ModalState, State};
use crate::tui::{Event, Tui};
use crate::{adb::client::AdbClient, panes::Pane};

pub struct App {
    state: State,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();
        let emulators = adb.avds_with_status(&devices);

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Ok(Self {
            state: State {
                running: true,
                should_suspend: false,
                focus: Pane::DeviceList,
                config,
                adb,
                last_refresh: Instant::now(),
                devices: DevicesState {
                    items: devices,
                    selected_index: 0,
                },
                emulators: EmulatorsState {
                    items: emulators,
                    selected_index: 0,
                },
                content: ContentState {},
                modal: ModalState::None,
            },
            action_tx: action_tx,
            action_rx: action_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;

            if self.state.should_suspend {
                tui.suspend()?;
                self.action_tx.send(Action::Resume)?;
                self.action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if !self.state.running {
                tui.stop()?;
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&self, key: KeyEvent) {
        if matches!(self.state.modal, ModalState::Help) {
            if let Some(Action::CloseModal | Action::ToggleHelp) = self.lookup_action(key) {
                let _ = self.action_tx.send(Action::CloseModal);
            }
            return;
        }

        if let Some(action) = self.lookup_action(key) {
            let _ = self.action_tx.send(action);
        }
    }

    fn lookup_action(&self, key: KeyEvent) -> Option<Action> {
        let key_seq = vec![key];
        self.state
            .config
            .keybindings
            .0
            .get(&self.state.focus)
            .and_then(|bindings: &HashMap<Vec<KeyEvent>, Action>| bindings.get(&key_seq))
            .cloned()
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            self.handle_action(action, tui)?;
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action, tui: &mut Tui) -> Result<()> {
        if action != Action::Tick && action != Action::Render {
            debug!("{action:?}");
        }

        match action {
            Action::Tick => {
                if self.state.last_refresh.elapsed() >= std::time::Duration::from_secs(2) {
                    self.action_tx.send(Action::RefreshDevices)?;
                }
            }
            Action::Quit => self.state.running = false,
            Action::Suspend => self.state.should_suspend = true,
            Action::Resume => self.state.should_suspend = false,
            Action::ClearScreen => tui.terminal.clear()?,
            Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
            Action::Render => self.render(tui)?,

            Action::CycleFocus => {
                self.state.focus = self.state.focus.next();
            }
            Action::RefreshDevices => {
                if let Ok(devices) = self.state.adb.devices() {
                    let emulators = self.state.adb.avds_with_status(&devices);
                    self.action_tx.send(Action::DevicesUpdated(devices))?;
                    self.action_tx.send(Action::EmulatorsUpdated(emulators))?;
                }
                self.state.last_refresh = Instant::now();
            }
            _ => {}
        }

        // Delegate to pane update functions and collect commands
        let mut commands = Vec::new();
        commands.extend(panes::devices::update(&mut self.state, &action));
        commands.extend(panes::emulators::update(&mut self.state, &action));
        commands.extend(panes::content::update(&mut self.state, &action));
        commands.extend(modals::help::update(&mut self.state, &action));

        self.execute_commands(commands)?;

        Ok(())
    }

    fn execute_commands(&mut self, commands: Vec<Command>) -> Result<()> {
        for cmd in commands {
            match cmd {
                Command::StartEmulator(name) => {
                    let _ = self.state.adb.start_emulator(&name);
                }
                Command::KillEmulator(serial) => {
                    let _ = self.state.adb.kill_emulator(&serial);
                }
                Command::Focus(panel) => {
                    self.state.focus = panel;
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| self.draw(frame))?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

        draw_title_bar(frame, vertical[0]);

        let middle = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(vertical[1]);

        let sidebar = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(middle[0]);

        panes::devices::draw(frame, sidebar[0], &self.state);
        panes::emulators::draw(frame, sidebar[1], &self.state);
        panes::content::draw(frame, middle[1], &self.state);
        draw_command_bar(frame, vertical[2], self.state.focus);
        modals::help::draw(frame, area, &self.state);
    }
}

fn draw_title_bar(frame: &mut Frame, area: Rect) {
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "LazyADB v0.1.0",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let device_span = Line::from(vec![Span::styled(
        "device: <none>",
        Style::default().fg(Color::DarkGray),
    )]);

    let device = Paragraph::new(device_span).right_aligned();

    frame.render_widget(title, columns[0]);
    frame.render_widget(device, columns[1]);
}

fn draw_command_bar(frame: &mut Frame, area: Rect, focus: Pane) {
    let columns = Layout::horizontal([Constraint::Min(0), Constraint::Length(8)]).split(area);

    let mut hints = vec![("q", "Quit"), ("Tab", "Focus"), ("j/k", "Select")];
    match focus {
        Pane::DeviceList => {
            hints.push(("r", "Refresh"));
        }
        Pane::Emulators => {
            hints.push(("r", "Refresh"));
            hints.push(("Enter", "Start"));
            hints.push(("x", "Kill"));
        }
        _ => {}
    }
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::White),
        ));
    }
    let left = Paragraph::new(Line::from(spans));
    frame.render_widget(left, columns[0]);

    let right = Paragraph::new(Line::from(vec![Span::styled(
        "? help",
        Style::default().fg(Color::DarkGray),
    )]))
    .right_aligned();
    frame.render_widget(right, columns[1]);
}
