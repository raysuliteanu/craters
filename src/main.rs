use anyhow::Result;
use crates_io_api::Summary;
use log::debug;
use ratatui::{
    layout::Flex,
    style::{Color, Modifier, Style},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};
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

use crate::crates_io_client::HttpClient;

// NOTE: order matters in this list for proper next/previous navigation
#[derive(Default, Debug, Copy, Clone, Display, FromRepr, EnumIter, PartialEq, Eq, Hash)]
enum SelectedSection {
    #[default]
    NewCrates,
    MostDownloaded,
    JustUpdated,
    RecentDownloads,
    PopularKeywords,
    PopularCategories,
}

mod crates_io_client;

#[allow(dead_code)]
pub struct App {
    client: crates_io_client::HttpClient,
    summary: Summary,
    current_section: SelectedSection,
    state: HashMap<SelectedSection, ListState>,
    exit: bool,
    search: bool,
    info: bool,
}

const SELECTED_STYLE: Style = Style::new().add_modifier(Modifier::BOLD).fg(Color::Blue);

impl App {
    async fn new() -> Self {
        let client = HttpClient::new();
        let mut state: HashMap<SelectedSection, ListState> = HashMap::new();
        for section in SelectedSection::iter() {
            let mut list_state = ListState::default();
            list_state.select(Some(0));
            state.insert(section, list_state);
        }
        let summary = client
            .fetch_summary()
            .await
            .expect("failed to fetch summary");

        App {
            client,
            current_section: SelectedSection::NewCrates,
            summary,
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
            if self.exit && self.info {
                self.info = false;
                self.exit = false;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let main = self.create_main_area();
        let inner_area = main.inner(frame.area());

        frame.render_widget(main, frame.area());

        let [row1, row2] = self.create_layout(inner_area);

        let new_crates_block = self.create_new_crates_area();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::NewCrates)
            .expect("selected section should always exist");
        frame.render_stateful_widget(new_crates_block, row1[0], state);

        let most_downloaded_block = self.create_most_downloaded();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::MostDownloaded)
            .expect("selected section should always exist");
        frame.render_stateful_widget(most_downloaded_block, row1[1], state);

        let just_updated_block = self.create_just_updated_area();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::JustUpdated)
            .expect("selected section should always exist");
        frame.render_stateful_widget(just_updated_block, row1[2], state);

        let recent_downloads_block = self.create_recent_downloads();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::RecentDownloads)
            .expect("selected section should always exist");
        frame.render_stateful_widget(recent_downloads_block, row2[0], state);

        let keyword_block = self.create_popular_keywords();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::PopularKeywords)
            .expect("selected section should always exist");
        frame.render_stateful_widget(keyword_block, row2[1], state);

        let categories_block = self.create_popular_categories();
        let state = &mut self
            .state
            .get_mut(&SelectedSection::PopularCategories)
            .expect("selected section should always exist");
        frame.render_stateful_widget(categories_block, row2[2], state);

        if self.info {
            self.info(frame);
        }
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

    fn create_new_crates_area(&self) -> List<'static> {
        self.create_block("New Crates", SelectedSection::NewCrates)
    }

    fn create_most_downloaded(&self) -> List<'static> {
        self.create_block("Most Downloaded", SelectedSection::MostDownloaded)
    }

    fn create_just_updated_area(&self) -> List<'static> {
        self.create_block("Just Updated", SelectedSection::JustUpdated)
    }

    fn create_recent_downloads(&self) -> List<'static> {
        self.create_block("Recent Downloads", SelectedSection::RecentDownloads)
    }

    fn create_popular_keywords(&self) -> List<'static> {
        self.create_block("Popular Keywords", SelectedSection::PopularKeywords)
    }

    fn create_popular_categories(&self) -> List<'static> {
        self.create_block("Popular Categories", SelectedSection::PopularCategories)
    }

    fn create_block(&self, title: &'static str, section: SelectedSection) -> List<'static> {
        let title = Line::from(title.bold().blue()).left_aligned();

        let border = if section == self.current_section {
            border::PLAIN
        } else {
            border::EMPTY
        };

        let block = Block::bordered().title(title).border_set(border);
        let crate_info = match section {
            SelectedSection::NewCrates => &self
                .summary
                .new_crates
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<&String>>(),
            SelectedSection::MostDownloaded => &self
                .summary
                .most_downloaded
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<&String>>(),
            SelectedSection::JustUpdated => &self
                .summary
                .just_updated
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<&String>>(),
            SelectedSection::RecentDownloads => &self
                .summary
                .most_recently_downloaded
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<&String>>(),
            SelectedSection::PopularKeywords => &self
                .summary
                .popular_keywords
                .iter()
                .map(|c| &c.keyword)
                .collect::<Vec<&String>>(),
            SelectedSection::PopularCategories => &self
                .summary
                .popular_categories
                .iter()
                .map(|c| &c.category)
                .collect::<Vec<&String>>(),
        };
        let list_items = crate_info
            .iter()
            .map(|s| ListItem::from(s.to_string()))
            .collect::<Vec<ListItem>>();
        List::new(list_items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
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
            KeyCode::Char('s') => self.search = true,
            KeyCode::Char('i') => self.info = true,
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

    fn info(&mut self, frame: &mut Frame) {
        let list_state = self
            .state
            .get(&self.current_section)
            .expect("sections always exist");
        let selected_index = list_state.selected().unwrap_or(0);
        let krate = match self.current_section {
            SelectedSection::NewCrates => self.summary.new_crates.get(selected_index).unwrap(),
            SelectedSection::MostDownloaded => {
                self.summary.most_downloaded.get(selected_index).unwrap()
            }
            SelectedSection::JustUpdated => self.summary.just_updated.get(selected_index).unwrap(),
            SelectedSection::RecentDownloads => self
                .summary
                .most_recently_downloaded
                .get(selected_index)
                .unwrap(),
            SelectedSection::PopularKeywords => todo!(),
            SelectedSection::PopularCategories => todo!(),
        };

        let block = Block::bordered().title(krate.name.clone());
        let paragraph = Paragraph::new(format!("{:?}", krate,))
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        let area = App::popup_area(frame.area(), 60, 60);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    #[allow(dead_code)]
    fn query(&mut self) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("craters.log").unwrap(),
    )
    .unwrap();
    let mut terminal = ratatui::init();
    let app_result = App::new().await.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_key_event() -> io::Result<()> {
        let mut app = App::new().await;

        assert_eq!(app.current_section, SelectedSection::NewCrates);

        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_section, SelectedSection::MostDownloaded);

        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_section, SelectedSection::JustUpdated);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_section, SelectedSection::MostDownloaded);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_section, SelectedSection::NewCrates);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_section, SelectedSection::PopularCategories);

        let mut app = App::new().await;
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
