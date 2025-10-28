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

    let mut app = App::new(0.0, 0.0).await?;
    app.run().await?;
    Ok(())
}
