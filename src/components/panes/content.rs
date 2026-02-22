use std::time::Instant;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    adb::device_info::DeviceInfo,
    command::Command,
    components::{Component, DrawContext, panes::Pane},
    msg::Msg,
};

const DEVICE_INFO_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

pub struct ContentPane {
    device_info: Option<DeviceInfo>,
    selected_serial: Option<String>,
    last_refresh: Instant,
}

impl ContentPane {
    pub fn new() -> Self {
        Self {
            device_info: None,
            selected_serial: None,
            last_refresh: Instant::now(),
        }
    }
}

impl Component for ContentPane {
    fn update(&mut self, action: &Msg) -> Vec<Command> {
        match action {
            Msg::Tick => {
                if let Some(ref serial) = self.selected_serial {
                    if self.last_refresh.elapsed() >= DEVICE_INFO_REFRESH_INTERVAL {
                        self.last_refresh = Instant::now();
                        return vec![Command::RefreshDeviceInfo(serial.clone())];
                    }
                }
            }
            Msg::DeviceSelected(device) => {
                let new_serial = device.as_ref().map(|d| d.serial.clone());
                if new_serial != self.selected_serial {
                    self.selected_serial = new_serial.clone();
                    self.device_info = None;
                    if let Some(serial) = new_serial {
                        self.last_refresh = Instant::now();
                        return vec![Command::RefreshDeviceInfo(serial)];
                    }
                }
            }
            Msg::DeviceInfoUpdated(info) => {
                if self.selected_serial.as_ref() == Some(&info.serial) {
                    self.device_info = Some(info.clone());
                }
            }
            _ => {}
        }
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

        let Some(ref info) = self.device_info else {
            let text = if self.selected_serial.is_some() {
                "Loading device info..."
            } else {
                "Select a device to begin"
            };
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, area);
            return;
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        // Model header
        lines.push(Line::from(vec![Span::styled(
            format!(" {} ", info.model),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Key-value rows
        let label_style = Style::default().fg(Color::DarkGray);
        let value_style = Style::default().fg(Color::White);

        let rows: Vec<(&str, String)> = vec![
            ("Serial", info.serial.clone()),
            ("Android", format!("{} (API {})", info.android_version, info.api_level)),
            ("State", info.state.clone()),
            ("Connection", info.connection_type.clone()),
            ("ABI", info.abi.clone()),
            ("Locale", info.locale.clone()),
        ];

        for (label, value) in &rows {
            lines.push(Line::from(vec![
                Span::styled(format!(" {:>12}  ", label), label_style),
                Span::styled(value.as_str(), value_style),
            ]));
        }

        // Screen
        if let Some(ref screen) = info.screen {
            lines.push(Line::from(vec![
                Span::styled(format!(" {:>12}  ", "Screen"), label_style),
                Span::styled(
                    format!("{} @ {}", screen.resolution, screen.density),
                    value_style,
                ),
            ]));
        }

        // Wi-Fi
        if let Some(ref wifi) = info.wifi {
            lines.push(Line::from(vec![
                Span::styled(format!(" {:>12}  ", "Wi-Fi"), label_style),
                Span::styled(format!("{} ({})", wifi.ssid, wifi.ip), value_style),
            ]));
        }

        lines.push(Line::from(""));

        // Battery bar
        if let Some(ref battery) = info.battery {
            lines.push(render_bar_line("Battery", battery.level as f64, 100.0, &battery.status, Color::Green));
        }

        // Storage bar
        if let Some(ref storage) = info.storage {
            let pct_label = format!("{:.1}/{:.1} GB", storage.used_gb, storage.total_gb);
            lines.push(render_bar_line("Storage", storage.used_gb, storage.total_gb, &pct_label, Color::Yellow));
        }

        // RAM bar
        if let Some(ref ram) = info.ram {
            let pct_label = format!("{:.1}/{:.1} GB", ram.used_gb, ram.total_gb);
            lines.push(render_bar_line("RAM", ram.used_gb, ram.total_gb, &pct_label, Color::Magenta));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn id(&self) -> &'static str {
        "Content"
    }
}

fn render_bar_line<'a>(
    label: &str,
    used: f64,
    total: f64,
    suffix: &str,
    color: Color,
) -> Line<'a> {
    let label_style = Style::default().fg(Color::DarkGray);
    let bar_width = 20;
    let ratio = if total > 0.0 { (used / total).clamp(0.0, 1.0) } else { 0.0 };
    let filled = (ratio * bar_width as f64).round() as usize;
    let empty = bar_width - filled;

    let bar_filled: String = "\u{2588}".repeat(filled);
    let bar_empty: String = "\u{2591}".repeat(empty);

    Line::from(vec![
        Span::styled(format!(" {:>12}  ", label), label_style),
        Span::styled(bar_filled, Style::default().fg(color)),
        Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {}", suffix),
            Style::default().fg(Color::White),
        ),
    ])
}
