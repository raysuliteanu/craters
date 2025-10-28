use std::collections::HashMap;

use color_eyre::Result;
use crates_io_api::{CrateResponse, Summary};
use crossterm::event::KeyCode;
use ratatui::{prelude::*, symbols::border, widgets::*};
use strum::IntoEnumIterator;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    app::{COLOR_BLACK, SelectedSection},
    config::Config,
    crates_io_client::HttpClient,
};

const LIST_ITEM_SELECTED_STYLE: Style = Style::new().add_modifier(Modifier::BOLD).fg(Color::Cyan);

pub struct Dashboard {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    client: HttpClient,
    summary: Summary,
    current_section: SelectedSection,
    state: HashMap<SelectedSection, ListState>,
    crates: HashMap<String, CrateResponse>,
}

impl Dashboard {
    pub async fn new() -> Self {
        let client = HttpClient::new();
        let summary = client
            .fetch_summary()
            .await
            .expect("failed to fetch summary");
        let mut state: HashMap<SelectedSection, ListState> = HashMap::new();
        for section in SelectedSection::iter() {
            let mut list_state = ListState::default();
            list_state.select(Some(0));
            state.insert(section, list_state);
        }
        Dashboard {
            client,
            summary,
            command_tx: None,
            config: Config::default(),
            current_section: SelectedSection::default(),
            state,
            crates: HashMap::new(),
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
                Constraint::Min(0), // Take remaining space
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
        let main_title = Line::from(" crates.io ".bold()).white().centered();

        let instructions = Line::from(vec![
            " Search ".into(),
            "s".blue().bold(),
            " Info ".into(),
            "i".blue().bold(),
            " Quit ".into(),
            "q ".blue().bold(),
        ]);

        Block::bordered()
            .title(main_title)
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .bg(COLOR_BLACK)
            .fg(Color::White)
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

        let block = Block::bordered()
            .title(title)
            .border_set(border)
            .bg(COLOR_BLACK);
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
            .style(Style::default().fg(Color::White))
            .highlight_style(LIST_ITEM_SELECTED_STYLE)
    }
}

impl Component for Dashboard {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            KeyCode::Char('i') => Some(Action::ShowInfo),
            KeyCode::Char('h') => Some(Action::Help),
            _ => None,
        };

        Ok(action)
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _area: Rect) -> Result<()> {
        let main = self.create_main_area();
        let inner_area = main.inner(frame.area());
        frame.render_widget(Clear, inner_area);
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

        Ok(())
    }
}
