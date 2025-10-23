use color_eyre::Result;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{collections::HashMap, fs::File, io};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::Block,
};
use strum::FromRepr;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{app::App, crates_io_client::HttpClient};

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
// async fn main() -> Result<()> {
//     WriteLogger::init(
//         LevelFilter::Debug,
//         Config::default(),
//         File::create("craters.log").unwrap(),
//     )
//     .unwrap();
//     let app_result = App::new().await.run().await;
//     ratatui::restore();
//     app_result
// }
