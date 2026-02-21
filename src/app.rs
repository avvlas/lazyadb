use std::collections::HashMap;
use std::time::Instant;

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

use crate::{
    action::Action,
    adb::client::AdbClient,
    command::Command,
    components::{
        Component, DrawContext,
        modals::{Modal, emulators::EmulatorsModal, help::HelpModal},
        panes::{Pane, content::ContentPane, devices::DevicesPane},
    },
    config::Config,
    tui::{Event, Tui},
};

pub struct App {
    running: bool,
    should_suspend: bool,
    focus: Pane,
    config: Config,
    adb: AdbClient,
    last_refresh: Instant,

    devices: DevicesPane,
    content: ContentPane,

    modal: Option<Modal>,

    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Ok(Self {
            running: true,
            should_suspend: false,
            focus: Pane::DeviceList,
            config,
            adb,
            last_refresh: Instant::now(),

            devices: DevicesPane::new(devices),
            content: ContentPane::new(),

            modal: None,

            action_tx,
            action_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;

            if self.should_suspend {
                debug!("Suspending app");
                tui.suspend()?;
                self.action_tx.send(Action::Resume)?;
                self.action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if !self.running {
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
        debug!("Handle key: {}", key.code);

        if let Some(ref modal) = self.modal {
            match modal {
                // Help modal: only allow closing
                Modal::Help(_) => {
                    if let Some(Action::CloseModal | Action::OpenHelp) = self.lookup_action(key) {
                        let _ = self.action_tx.send(Action::CloseModal);
                    }
                    return;
                }

                // Emulators modal: route keys through Emulators keybindings
                Modal::Emulators(_) => {
                    if let Some(action) = self.lookup_emulator_action(key) {
                        match action {
                            Action::EmulatorListUp
                            | Action::EmulatorListDown
                            | Action::EmulatorSelect
                            | Action::KillEmulator
                            | Action::CloseModal => {
                                let _ = self.action_tx.send(action);
                            }
                            _ => {}
                        }
                    }
                    return;
                }
            }
        }

        if let Some(action) = self.lookup_action(key) {
            let _ = self.action_tx.send(action);
        }
    }

    fn lookup_action(&self, key: KeyEvent) -> Option<Action> {
        let key_seq = vec![key];
        self.config
            .keybindings
            .0
            .get(&self.focus)
            .and_then(|bindings: &HashMap<Vec<KeyEvent>, Action>| bindings.get(&key_seq))
            .cloned()
    }

    fn lookup_emulator_action(&self, key: KeyEvent) -> Option<Action> {
        let key_seq = vec![key];
        self.config
            .keybindings
            .0
            .get(&Pane::Emulators)
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
            debug!("Handling action: {action:?}");
        }

        match action {
            Action::Tick => {
                if self.last_refresh.elapsed() >= std::time::Duration::from_secs(2) {
                    self.action_tx.send(Action::RefreshDevices)?;
                }
            }
            Action::Quit => self.running = false,
            Action::Suspend => self.should_suspend = true,
            Action::Resume => self.should_suspend = false,
            Action::ClearScreen => tui.terminal.clear()?,
            Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
            Action::Render => self.render(tui)?,

            Action::CycleFocus => {
                self.focus = self.focus.next();
            }
            Action::CycleFocusBackwards => {
                self.focus = self.focus.prev();
            }

            Action::OpenHelp => self.modal = Some(Modal::Help(HelpModal::new())),
            Action::CloseModal => self.modal = None,
            _ => {}
        }

        // Delegate to component update methods and collect commands
        let mut commands = Vec::new();
        for component in self.components() {
            commands.extend(component.update(&action));
        }

        self.execute_commands(commands)?;

        Ok(())
    }

    fn components(&mut self) -> Vec<&mut dyn Component> {
        let mut components: Vec<&mut dyn Component> = vec![&mut self.devices, &mut self.content];

        if let Some(ref mut modal) = self.modal {
            match modal {
                Modal::Help(help) => components.push(help),
                Modal::Emulators(emulators) => components.push(emulators),
            }
        }

        components
    }

    fn execute_commands(&mut self, commands: Vec<Command>) -> Result<()> {
        for cmd in commands {
            match cmd {
                Command::StartEmulator(name) => {
                    let _ = self.adb.start_emulator(&name);
                }
                Command::KillEmulator(serial) => {
                    let _ = self.adb.kill_emulator(&serial);
                }
                Command::OpenEmulatorsModal => {
                    let emulators = self.adb.avds_with_status(&self.devices.items);
                    self.modal = Some(Modal::Emulators(EmulatorsModal::new(emulators)))
                }
                Command::CloseEmulatorsModal => self.modal = None,
                Command::DisconnectDevice(serial) => {
                    let _ = self.adb.disconnect_device(&serial);
                }
                Command::Focus(panel) => {
                    self.focus = panel;
                }
                Command::CycleFocus => {
                    self.focus = self.focus.next();
                }
                Command::RefreshDevices => {
                    if let Ok(devices) = self.adb.devices() {
                        self.action_tx.send(Action::DevicesUpdated(devices))?;
                        self.last_refresh = Instant::now();
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        debug!("Resizing: {} x {}", w, h);

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

        let ctx = DrawContext {
            focus: self.focus,
            config: &self.config,
        };

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

        draw_title_bar(frame, vertical[0]);

        let middle = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(vertical[1]);

        self.devices.draw(frame, middle[0], &ctx);
        self.content.draw(frame, middle[1], &ctx);
        draw_command_bar(frame, vertical[2], self.focus);

        if let Some(ref modal) = self.modal {
            match modal {
                Modal::Help(help) => help.draw(frame, area, &ctx),
                Modal::Emulators(emulators) => emulators.draw(frame, area, &ctx),
            }
        }
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
            hints.push(("x", "Disconnect"));
            hints.push(("e", "Emulators"));
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
