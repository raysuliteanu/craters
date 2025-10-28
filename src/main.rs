use color_eyre::Result;

use crate::app::App;

mod action;
mod app;
mod components;
mod config;
mod crates_io_client;
mod errors;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    // 4 and 60 values come from the ratatui template, originally from the
    // CLI module which I didn't include
    let mut app = App::new(4.0, 60.0).await?;
    app.run().await?;
    Ok(())
}
