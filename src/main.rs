use anyhow::Result;
use crates_io_api::{CrateResponse, Summary};
use log::debug;
use ratatui::{
    layout::Flex,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap},
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
    client: HttpClient,
    summary: Summary,
    current_section: SelectedSection,
    state: HashMap<SelectedSection, ListState>,
    crates: HashMap<String, CrateResponse>,
    should_exit: bool,
    search: bool,
    show_info_popup: bool,
}

const LIST_ITEM_SELECTED_STYLE: Style = Style::new().add_modifier(Modifier::BOLD).fg(Color::Blue);

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
            crates: HashMap::new(),
            should_exit: false,
            search: false,
            show_info_popup: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;

            self.handle_events().await?;

            // user pressed 'q' or 'esc' while info popup was open, so make sure we only close the
            // popup, and don't exit the app
            if self.should_exit && self.show_info_popup {
                self.show_info_popup = false;
                self.should_exit = false;
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

        if self.show_info_popup {
            self.render_info_popup(frame);
        }
    }

    fn create_layout(&self, area: Rect) -> [[Rect; 3]; 2] {
        // Each block needs: 10 lines (content) + 2 lines (borders) + 1 line (title) = 13 lines
        let block_height = 13;

        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(block_height),
                Constraint::Min(block_height),
                Constraint::Min(0),  // Take remaining space
            ])
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
            .highlight_style(LIST_ITEM_SELECTED_STYLE)
    }

    /// updates the application's state based on user input
    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => self.search = true,
            KeyCode::Char('i') => self.show_info().await,
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => self.next_section(),
            KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => self.previous_section(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }

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
                _ => unreachable!(),
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

    fn render_info_popup(&mut self, frame: &mut Frame) {
        let list_state = self
            .state
            .get(&self.current_section)
            .expect("sections always exist");
        let selected_index = list_state.selected().unwrap_or(0);
        let lines = match self.current_section {
            SelectedSection::NewCrates => {
                let krate = self.summary.new_crates.get(selected_index).unwrap();
                self.format_crate_info(&krate.name)
            }
            SelectedSection::MostDownloaded => {
                let krate = self.summary.most_downloaded.get(selected_index).unwrap();
                self.format_crate_info(&krate.name)
            }
            SelectedSection::JustUpdated => {
                let krate = self.summary.just_updated.get(selected_index).unwrap();
                self.format_crate_info(&krate.name)
            }
            SelectedSection::RecentDownloads => {
                let krate = self
                    .summary
                    .most_recently_downloaded
                    .get(selected_index)
                    .unwrap();
                self.format_crate_info(&krate.name)
            }
            // SelectedSection::PopularKeywords => None,
            // SelectedSection::PopularCategories => None,
            _ => todo!("info for other sections not implemented yet"),
        };

        let block = Block::bordered();
        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        let area = App::popup_area(frame.area(), 60, 60);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }

    fn format_crate_info(&self, crate_name: &str) -> Vec<Line<'static>> {
        // Try to get full crate info from cache, fallback to empty if not available
        if let Some(crate_response) = self.crates.get(crate_name) {
            let krate = &crate_response.crate_data;
            let desc = krate
                .description
                .clone()
                .unwrap_or_else(|| "No description".into());
            let tags = krate
                .keywords
                .as_ref()
                .map(|keywords| {
                    keywords
                        .iter()
                        .map(|k| format!("#{}", k))
                        .collect::<Vec<String>>()
                        .join("\t")
                })
                .unwrap_or_default();
            let name = Span::from(krate.name.clone()).bold();
            let version = Span::from(krate.max_version.clone());
            vec![
                Line::from(vec![name, Span::from("    "), version]),
                Line::from(""),
                Line::from(desc),
                Line::from(""),
                Line::from(tags.green()),
            ]
        } else {
            // Fallback if crate info not yet loaded
            vec![Line::from("Loading crate information...")]
        }
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
    let app_result = App::new().await.run(&mut terminal).await;
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

        app.handle_key_event(KeyCode::Right.into()).await;
        assert_eq!(app.current_section, SelectedSection::MostDownloaded);

        app.handle_key_event(KeyCode::Right.into()).await;
        assert_eq!(app.current_section, SelectedSection::JustUpdated);

        app.handle_key_event(KeyCode::Left.into()).await;
        assert_eq!(app.current_section, SelectedSection::MostDownloaded);

        app.handle_key_event(KeyCode::Left.into()).await;
        assert_eq!(app.current_section, SelectedSection::NewCrates);

        app.handle_key_event(KeyCode::Left.into()).await;
        assert_eq!(app.current_section, SelectedSection::PopularCategories);

        let mut app = App::new().await;
        app.handle_key_event(KeyCode::Char('q').into()).await;
        assert!(app.should_exit);

        Ok(())
    }
}
