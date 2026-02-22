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
    adb::client::AdbClient,
    command::Command,
    components::{
        Component, DrawContext,
        modals::{Modal, emulators::EmulatorsModal, help::HelpModal},
        panes::{Pane, content::ContentPane, devices::DevicesPane},
    },
    config::Config,
    msg::Msg,
    tui::{Event, Tui},
};

pub struct App {
    running: bool,

    focus: Pane,
    config: Config,
    adb: AdbClient,
    last_refresh: Instant,

    devices: DevicesPane,
    content: ContentPane,

    modal: Option<Modal>,

    msg_tx: mpsc::UnboundedSender<Msg>,
    msg_rx: mpsc::UnboundedReceiver<Msg>,
}

enum GlobalAction {
    Quit,
    CycleFocus,
    CycleFocusBackwards,
    ToggleHelp,
    CloseModal,
}

impl GlobalAction {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Quit" => Some(Self::Quit),
            "CycleFocus" => Some(Self::CycleFocus),
            "CycleFocusBackwards" => Some(Self::CycleFocusBackwards),
            "ToggleHelp" => Some(Self::ToggleHelp),
            "CloseModal" => Some(Self::CloseModal),
            _ => None,
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();

        let device_keymap = config.keybindings.section_keymap("DeviceList");
        let devices_pane = DevicesPane::new(devices, device_keymap);

        let (msg_tx, msg_rx) = mpsc::unbounded_channel();

        Ok(Self {
            running: true,

            focus: Pane::DeviceList,
            config: config,
            adb: adb,
            last_refresh: Instant::now(),

            devices: devices_pane,
            content: ContentPane::new(),

            modal: None,

            msg_tx,
            msg_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions()?;

            if !self.running {
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
        match event {
            Event::Init => {}
            Event::Tick => self.msg_tx.send(Msg::Tick)?,
            Event::Resize(w, h) => self.handle_resize(tui, w, h)?,
            Event::Render => self.render(tui)?,
            Event::Key(key) => self.handle_key(key),
            Event::Error => {}
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Paste(_) => {}
            Event::Mouse(_) => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        debug!("Handle key: {}", key.code);

        let global_action = self.lookup_global_action(key);

        // If a modal is open, handle it specially
        if let Some(ref mut modal) = self.modal {
            match modal {
                Modal::Help(help) => {
                    // Help modal: only allow closing via global keys
                    if let Some(action) = global_action {
                        match action {
                            GlobalAction::CloseModal | GlobalAction::ToggleHelp => {
                                self.modal = None;
                                return;
                            }
                            GlobalAction::Quit => {
                                self.running = false;
                                return;
                            }
                            _ => {}
                        }
                    }
                    let commands = help.update(&Msg::KeyPress(key));
                    self.execute_commands(commands).ok();
                    return;
                }
                Modal::Emulators(emulators) => {
                    // Check global keys first
                    if let Some(action) = global_action {
                        match action {
                            GlobalAction::CloseModal => {
                                self.modal = None;
                                return;
                            }
                            GlobalAction::ToggleHelp => {
                                self.modal = Some(Modal::Help(HelpModal::new()));
                                return;
                            }
                            GlobalAction::Quit => {
                                self.running = false;
                                return;
                            }
                            _ => {}
                        }
                    }
                    // Forward to modal's handle_key
                    let commands = emulators.update(&Msg::KeyPress(key));
                    self.execute_commands(commands).ok();
                    return;
                }
            }
        }

        // Check global keybindings
        if let Some(action) = global_action {
            self.handle_global_action(action);
            return;
        }

        // Forward to focused component
        let commands = self.focused_pane().update(&Msg::KeyPress(key));

        self.execute_commands(commands).ok();
    }

    fn handle_global_action(&mut self, action: GlobalAction) {
        match action {
            GlobalAction::Quit => {
                self.running = false;
            }
            GlobalAction::CycleFocus => {
                self.focus = self.focus.next();
            }
            GlobalAction::CycleFocusBackwards => {
                self.focus = self.focus.prev();
            }
            GlobalAction::ToggleHelp => {
                self.modal = Some(Modal::Help(HelpModal::new()));
            }
            GlobalAction::CloseModal => {
                self.modal = None;
            }
        }
    }

    fn lookup_global_action(&self, key: KeyEvent) -> Option<GlobalAction> {
        self.config
            .keybindings
            .lookup_global(&key)
            .and_then(GlobalAction::from_str)
    }

    fn handle_actions(&mut self) -> Result<()> {
        while let Ok(action) = self.msg_rx.try_recv() {
            if !matches!(action, Msg::Tick) {
                debug!("Handling action: {action:?}");
            }

            // Delegate to component update methods and collect commands
            let mut commands = Vec::new();
            for component in self.components() {
                commands.extend(component.update(&action));
            }

            self.execute_commands(commands)?;
        }
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

    fn focused_pane(&mut self) -> &mut dyn Component {
        match self.focus {
            Pane::DeviceList => &mut self.devices,
            Pane::Content => &mut self.content,
        }
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
                    let keymap = self.config.keybindings.section_keymap("EmulatorsModal");
                    self.modal = Some(Modal::Emulators(EmulatorsModal::new(emulators, keymap)))
                }
                Command::CloseEmulatorsModal => self.modal = None,
                Command::DisconnectDevice(serial) => {
                    let _ = self.adb.disconnect_device(&serial);
                }
                Command::Focus(panel) => {
                    self.focus = panel;
                }
                Command::RefreshDevices => {
                    if let Ok(devices) = self.adb.devices() {
                        self.msg_tx.send(Msg::DevicesUpdated(devices))?;
                        self.last_refresh = Instant::now();
                    }
                }
                Command::RefreshDeviceInfo(serial) => {
                    if let Some(device) = self.devices.items.iter().find(|d| d.serial == serial) {
                        if let Ok(info) = self.adb.fetch_device_info(device) {
                            self.msg_tx.send(Msg::DeviceInfoUpdated(info))?;
                        }
                    }
                }
                Command::DeviceSelected(device) => {
                    self.msg_tx.send(Msg::DeviceSelected(device))?;
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

        let selected_device_name = self
            .devices
            .selected_device()
            .map(|d| d.display_name());
        draw_title_bar(frame, vertical[0], selected_device_name.as_deref());

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

fn draw_title_bar(frame: &mut Frame, area: Rect, selected_device: Option<&str>) {
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "LazyADB v0.1.0",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    let (device_text, device_color) = match selected_device {
        Some(name) => (format!("device: {}", name), Color::White),
        None => ("device: <none>".to_string(), Color::DarkGray),
    };

    let device_span = Line::from(vec![Span::styled(
        device_text,
        Style::default().fg(device_color),
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
