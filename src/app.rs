use std::collections::HashMap;

use crate::{
    action::Action,
    components::{Component, dashboard::Dashboard},
    config::Config,
    crates_io_client::HttpClient,
    tui::{Event, Tui},
};
use color_eyre::Result;
use crates_io_api::{CrateResponse, Summary};
use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, style::Color, widgets::ListState};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::mpsc;
use tracing::{debug, trace};

// TODO: move this to dashboard component, or make a lib.rs for shared stuff
pub(crate) const COLOR_BLACK: Color = Color::Rgb(0, 0, 0);

#[expect(dead_code)]
pub struct App {
    // TODO: probably can remove these two rates
    tick_rate: f64,
    frame_rate: f64,
    // TODO: probably don't need this
    should_suspend: bool,

    config: Config,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    client: HttpClient,
    summary: Summary,
    current_section: SelectedSection,
    state: HashMap<SelectedSection, ListState>,
    crates: HashMap<String, CrateResponse>,
    search: bool,
    show_info_popup: bool,
}

// NOTE: order matters in this list for proper next/previous navigation
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    Display,
    FromRepr,
    EnumIter,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub(crate) enum SelectedSection {
    #[default]
    NewCrates,
    MostDownloaded,
    JustUpdated,
    RecentDownloads,
    PopularKeywords,
    PopularCategories,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Dashboard,
    Search,
    Info,
}

impl App {
    pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut state: HashMap<SelectedSection, ListState> = HashMap::new();
        for section in SelectedSection::iter() {
            let mut list_state = ListState::default();
            list_state.select(Some(0));
            state.insert(section, list_state);
        }
        let client = HttpClient::new();
        let summary = client
            .fetch_summary()
            .await
            .expect("failed to fetch summary");
        let dashboard = Dashboard::new().await;
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(dashboard)],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Dashboard,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            client,
            current_section: SelectedSection::NewCrates,
            summary,
            state,
            crates: HashMap::new(),
            search: false,
            show_info_popup: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
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
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };

        match keymap.get(&vec![key]) {
            Some(action) => {
                trace!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    trace!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }

            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                _ => {}
            }

            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}

impl App {
    async fn show_info(&mut self) {
        // Only fetch for crate sections, not keywords/categories
        if matches!(
            self.current_section,
            SelectedSection::NewCrates
                | SelectedSection::MostDownloaded
                | SelectedSection::JustUpdated
                | SelectedSection::RecentDownloads
        ) {
            let list_state = self
                .state
                .get(&self.current_section)
                .expect("sections always exist");
            let selected_index = list_state.selected().unwrap_or(0);

            // Get the crate name from summary
            let crate_name = match self.current_section {
                SelectedSection::NewCrates => {
                    &self.summary.new_crates.get(selected_index).unwrap().name
                }
                SelectedSection::MostDownloaded => {
                    &self
                        .summary
                        .most_downloaded
                        .get(selected_index)
                        .unwrap()
                        .name
                }
                SelectedSection::JustUpdated => {
                    &self.summary.just_updated.get(selected_index).unwrap().name
                }
                SelectedSection::RecentDownloads => {
                    &self
                        .summary
                        .most_recently_downloaded
                        .get(selected_index)
                        .unwrap()
                        .name
                }
                _ => todo!("keywords and categories"),
            };

            if !self.crates.contains_key(crate_name)
                && let Ok(crate_response) = self.client.fetch_crate_info(crate_name).await
            {
                self.crates.insert(crate_name.clone(), crate_response);
            }
        }

        self.show_info_popup = true;
    }

    fn select_next(&mut self) {
        debug!("current_section: {}", self.current_section);
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .select_next();
    }

    fn select_previous(&mut self) {
        debug!("current_section: {}", self.current_section);
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .select_previous();
    }

    #[allow(dead_code)]
    fn select_none(&mut self) {
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .select(None);
    }

    #[allow(dead_code)]
    fn select_first(&mut self) {
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .select_first();
    }

    fn next_section(&mut self) {
        let mut next = self.current_section as usize + 1;
        if next >= SelectedSection::iter().count() {
            next = 0;
        }
        self.current_section = SelectedSection::from_repr(next).unwrap_or_default();
    }

    fn previous_section(&mut self) {
        let previous = if self.current_section as isize - 1 < 0 {
            SelectedSection::iter().count() - 1
        } else {
            self.current_section as usize - 1
        };
        self.current_section = SelectedSection::from_repr(previous).unwrap_or_default();
    }

    #[allow(dead_code)]
    fn query(&mut self) {}
}
