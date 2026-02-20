mod action;
mod adb;
mod app;
mod command;
mod config;
mod logging;
mod panes;
mod modals;
mod state;
mod tui;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    logging::init()?;

    let mut app = app::App::new()?;
    app.run().await?;
    Ok(())
}
