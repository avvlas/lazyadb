mod adb;
mod app;
mod command;
mod components;
mod config;
mod msg;
mod tui;

use color_eyre::Result;

use crate::config::logging;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    logging::init()?;

    let mut app = app::App::new()?;
    app.run().await?;
    Ok(())
}
