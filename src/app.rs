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
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::debug;

use crate::action::Action;
use crate::adb::client::AdbClient;
use crate::component::Component;
use crate::component::content_area::ContentAreaComponent;
use crate::component::device_list::DeviceListComponent;
use crate::component::emulator_list::EmulatorListComponent;
use crate::component::help_modal::HelpModalComponent;
use crate::config::Config;
use crate::tui::{Event, Tui};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FocusPanel {
    #[default]
    DeviceList,
    Emulators,
    Content,
}

impl FocusPanel {
    pub fn next(&self) -> Self {
        match self {
            Self::DeviceList => Self::Emulators,
            Self::Emulators => Self::Content,
            Self::Content => Self::DeviceList,
        }
    }
}

pub struct App {
    running: bool,
    should_suspend: bool,
    focus: FocusPanel,
    config: Config,
    adb: AdbClient,
    last_refresh: Instant,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    device_list: DeviceListComponent,
    emulator_list: EmulatorListComponent,
    content_area: ContentAreaComponent,
    help_modal: HelpModalComponent,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        let adb = AdbClient::new()?;
        let devices = adb.devices().unwrap_or_default();
        let emulators = adb.avds_with_status(&devices);

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let device_list = DeviceListComponent::new(devices);
        let emulator_list = EmulatorListComponent::new(emulators);
        let content_area = ContentAreaComponent::new();
        let help_modal = HelpModalComponent::new();

        Ok(Self {
            running: true,
            should_suspend: false,
            focus: FocusPanel::DeviceList,
            config,
            adb,
            last_refresh: Instant::now(),
            action_tx,
            action_rx,
            device_list,
            emulator_list,
            content_area,
            help_modal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        for component in self.components() {
            component.init(tui.size()?)?;
        }

        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;

            if self.should_suspend {
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

    async fn handle_events(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
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
        for component in self.components() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn components(&mut self) -> [&mut dyn Component; 4] {
        [
            &mut self.device_list,
            &mut self.emulator_list,
            &mut self.content_area,
            &mut self.help_modal,
        ]
    }

    fn handle_key(&self, key: KeyEvent) {
        // Modal override: when help is visible, only allow closing it
        if self.help_modal.is_visible() {
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
        self.config
            .keybindings
            .0
            .get(&self.focus)
            .and_then(|bindings| bindings.get(&key_seq))
            .cloned()
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
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
                self.action_tx.send(Action::FocusChanged(self.focus))?;
            }
            Action::RefreshDevices => {
                if let Ok(devices) = self.adb.devices() {
                    let emulators = self.adb.avds_with_status(&devices);
                    self.action_tx.send(Action::DevicesUpdated(devices))?;
                    self.action_tx.send(Action::EmulatorsUpdated(emulators))?;
                }
                self.last_refresh = Instant::now();
            }
            Action::StartEmulatorByName(name) => {
                let _ = self.adb.start_emulator(&name);
                return Ok(());
            }
            Action::KillEmulatorBySerial(serial) => {
                let _ = self.adb.kill_emulator(&serial);
                return Ok(());
            }
            Action::FocusChanged(panel) => {
                self.focus = panel;
            }
            _ => {}
        }

        // Broadcast to all components
        self.broadcast_action(action)?;

        Ok(())
    }

    fn broadcast_action(&mut self, action: Action) -> Result<()> {
        let action_tx = self.action_tx.clone();
        for component in self.components() {
            if let Some(fu) = component.update(action.clone())? {
                action_tx.send(fu)?;
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> color_eyre::Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        tui.draw(|frame| self.draw(frame))?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Vertical: title bar | middle | command bar
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

        draw_title_bar(frame, vertical[0]);

        // Middle: sidebar (20%) | content (80%)
        let middle = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(vertical[1]);

        // Sidebar: devices (top 50%) | emulators (bottom 50%)
        let sidebar = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(middle[0]);

        let _ = self.device_list.draw(frame, sidebar[0]);
        let _ = self.emulator_list.draw(frame, sidebar[1]);
        let _ = self.content_area.draw(frame, middle[1]);
        draw_command_bar(frame, vertical[2], self.focus);
        let _ = self.help_modal.draw(frame, area);
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

fn draw_command_bar(frame: &mut Frame, area: Rect, focus: FocusPanel) {
    let columns = Layout::horizontal([Constraint::Min(0), Constraint::Length(8)]).split(area);

    let mut hints = vec![("q", "Quit"), ("Tab", "Focus"), ("j/k", "Select")];
    match focus {
        FocusPanel::DeviceList => {
            hints.push(("r", "Refresh"));
        }
        FocusPanel::Emulators => {
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
