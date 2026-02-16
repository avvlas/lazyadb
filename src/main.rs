mod app;
mod event;
mod keys;
mod ui;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = app::App::new().run(&mut terminal);
    ratatui::restore();
    result
}
