use anyhow::Result;
use log::debug;
use ratatui::widgets::ListState;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{collections::HashMap, fs::File, io};

#[allow(unused_imports)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
#[allow(unused_imports)]
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use strum::FromRepr;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::http_client::CrateInfo;

// NOTE: order matters in this list for proper next/previous navigation
#[derive(Default, Debug, Copy, Clone, Display, FromRepr, EnumIter, PartialEq, Eq, Hash)]
enum SelectedSection {
    #[default]
    NewCrates,
    JustUpdated,
    MostDownloaded,
    RecentDownloads,
    Keywords,
    Categories,
}

mod http_client;

#[allow(dead_code)]
pub struct App {
    client: http_client::HttpClient,
    current_section: SelectedSection,
    state: HashMap<SelectedSection, CrateList>,
    exit: bool,
    search: bool,
    info: bool,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
struct CrateList {
    crates: Vec<CrateInfo>,
    state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    fn new() -> Self {
        let client = http_client::HttpClient::new();
        let mut state: HashMap<SelectedSection, CrateList> = HashMap::new();
        for section in SelectedSection::iter() {
            let crate_infos = match section {
                SelectedSection::NewCrates => client
                    .fetch_new_crates()
                    .expect("failed to fetch new crates"),
                SelectedSection::JustUpdated => client
                    .fetch_recent_updates()
                    .expect("failed to fetch recent updates"),
                SelectedSection::MostDownloaded => client
                    .fetch_most_downloaded()
                    .expect("failed to fetch most downloaded"),
                _ => vec![CrateInfo::default()], // Placeholder for unimplemented sections
            };

            state.insert(
                section,
                CrateList {
                    crates: crate_infos,
                    state: ListState::default(),
                },
            );
        }

        App {
            client,
            current_section: SelectedSection::NewCrates,
            state,
            exit: false,
            search: false,
            info: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let main = self.create_main_area();
        let inner_area = main.inner(frame.area());

        frame.render_widget(main, frame.area());

        let [row1, row2] = self.create_layout(inner_area);

        let new_crates_block = self.create_new_crates_area();

        frame.render_widget(
            self.placeholder_paragraph().block(new_crates_block),
            row1[0],
        );

        let just_updated_block = self.create_just_updated_area();

        frame.render_widget(
            self.placeholder_paragraph().block(just_updated_block),
            row1[1],
        );

        let most_downloaded_block = self.create_most_downloaded();

        frame.render_widget(
            self.placeholder_paragraph().block(most_downloaded_block),
            row1[2],
        );

        let recent_downloads_block = self.create_recent_downloads();

        frame.render_widget(
            self.placeholder_paragraph().block(recent_downloads_block),
            row2[0],
        );

        let keyword_block = self.create_popular_keywords();

        frame.render_widget(self.placeholder_paragraph().block(keyword_block), row2[1]);

        let categories_block = self.create_popular_categories();

        frame.render_widget(
            self.placeholder_paragraph().block(categories_block),
            row2[2],
        );
    }

    fn create_layout(&self, area: Rect) -> [[Rect; 3]; 2] {
        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let row1 = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(rows[0]);

        let row2 = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(rows[1]);

        [[row1[0], row1[1], row1[2]], [row2[0], row2[1], row2[2]]]
    }

    fn create_main_area(&self) -> Block<'static> {
        let main_title = Line::from(" crates.io ".bold());

        let instructions = Line::from(vec![
            " Search ".into(),
            "s".blue().bold(),
            " Info ".into(),
            "i".blue().bold(),
            " Quit ".into(),
            "q ".blue().bold(),
        ]);

        Block::bordered()
            .title(main_title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
    }

    fn create_new_crates_area(&self) -> Block<'static> {
        self.create_block(
            "New Crates",
            self.current_section == SelectedSection::NewCrates,
        )
    }

    fn create_just_updated_area(&self) -> Block<'static> {
        self.create_block(
            "Just Updated",
            self.current_section == SelectedSection::JustUpdated,
        )
    }

    fn create_most_downloaded(&self) -> Block<'static> {
        self.create_block(
            "Most Downloaded",
            self.current_section == SelectedSection::MostDownloaded,
        )
    }

    fn create_recent_downloads(&self) -> Block<'static> {
        self.create_block(
            "Most Recent Downloads",
            self.current_section == SelectedSection::RecentDownloads,
        )
    }

    fn create_popular_keywords(&self) -> Block<'static> {
        self.create_block(
            "Popular Keywords",
            self.current_section == SelectedSection::Keywords,
        )
    }

    fn create_popular_categories(&self) -> Block<'static> {
        self.create_block(
            "Popular Categories",
            self.current_section == SelectedSection::Categories,
        )
    }

    fn create_block(&self, title: &'static str, selected: bool) -> Block<'static> {
        let title = Line::from(title.bold().blue()).left_aligned();
        let border = if selected {
            border::PLAIN
        } else {
            border::EMPTY
        };

        Block::bordered().title(title).border_set(border)
    }

    fn placeholder_paragraph(&self) -> Paragraph<'static> {
        let crate_info = self
            .client
            .fetch_new_crates()
            .expect("failed to fetch from crates.io");
        let text = crate_info
            .iter()
            .map(|c| Line::from(c.name.clone()))
            .collect::<Vec<Line>>();
        Paragraph::new(text)
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => self.query(),
            KeyCode::Char('i') => self.info(),
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => self.next_section(),
            KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => self.previous_section(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next(&mut self) {
        debug!("current_section: {}", self.current_section);
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .state
            .select_next();
    }

    fn select_previous(&mut self) {
        debug!("current_section: {}", self.current_section);
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .state
            .select_previous();
    }

    #[allow(dead_code)]
    fn select_none(&mut self) {
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .state
            .select(None);
    }

    #[allow(dead_code)]
    fn select_first(&mut self) {
        self.state
            .get_mut(&self.current_section)
            .expect("should always have a section selected")
            .state
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
        let mut previous = self.current_section as usize;
        if self.current_section as isize - 1 < 0 {
            previous = SelectedSection::iter().count() - 1;
        }
        self.current_section = SelectedSection::from_repr(previous).unwrap_or_default();
    }

    fn info(&mut self) {}

    fn query(&mut self) {}
}

fn main() -> Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("craters.log").unwrap(),
    )
    .unwrap();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());

        app.handle_key_event(KeyCode::Left.into());

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
