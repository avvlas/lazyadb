mod action;
mod adb;
mod app;
mod config;
mod event;
mod keys;
mod logging;
mod ui;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    logging::init()?;

    let mut terminal = ratatui::init();
    let result = app::App::new()?.run(&mut terminal);
    ratatui::restore();
    result
}
