use color_eyre::Result;
use std::io;

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
use strum::{Display, EnumIter};

#[derive(Default, Debug, Copy, Clone, Display, FromRepr, EnumIter, PartialEq, Eq)]
enum SelectedSection {
    #[default]
    NewCrates,
    MostDownloaded,
    JustUpdated,
    RecentDownloads,
    Keywords,
    Categories,
}

#[derive(Debug, Default)]
pub struct App {
    current_section: SelectedSection,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let title_block = self.create_title_block();
        let inner_area = title_block.inner(frame.area());

        frame.render_widget(title_block, frame.area());

        let layout = self.create_layout(inner_area);

        let new_crates_title = Line::from(" New Crates ".bold());
        let new_crates_block = Block::bordered()
            .title(new_crates_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(new_crates_block),
            layout[0][0],
        );

        let just_updated_title = Line::from(" Just Updated ".bold());
        let just_updated_block = Block::bordered()
            .title(just_updated_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(just_updated_block),
            layout[0][1],
        );

        let most_downloaded_title = Line::from(" Most Downloaded ".bold());
        let most_downloaded_block = Block::bordered()
            .title(most_downloaded_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(most_downloaded_block),
            layout[0][2],
        );

        let recent_downloads_title = Line::from(" Most Recent Downloads ".bold());
        let recent_downloads_block = Block::bordered()
            .title(recent_downloads_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(recent_downloads_block),
            layout[1][0],
        );

        let keyword_title = Line::from(" Popular Keywords ".bold());
        let keyword_block = Block::bordered()
            .title(keyword_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(keyword_block),
            layout[1][1],
        );

        let categories_title = Line::from(" Popular Categories ".bold());
        let categories_block = Block::bordered()
            .title(categories_title.centered())
            .border_set(border::EMPTY);

        frame.render_widget(
            App::placeholder_paragraph().block(categories_block),
            layout[1][2],
        );
    }

    fn create_layout(&self, area: Rect) -> [[Rect; 3]; 2] {
        // Split the frame into 2 vertical rows (50% each)
        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Split the first row into 3 horizontal columns
        let row1 = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(rows[0]);

        // Split the second row into 3 horizontal columns
        let row2 = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(rows[1]);

        // Return as a 2x3 array
        [[row1[0], row1[1], row1[2]], [row2[0], row2[1], row2[2]]]
    }

    fn create_title_block(&self) -> Block<'static> {
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

    fn placeholder_paragraph() -> Paragraph<'static> {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        Paragraph::new(text.dark_gray()).wrap(Wrap { trim: true })
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
            // KeyCode::Char('l') | KeyCode::Right => self.next(),
            // KeyCode::Char('h') | KeyCode::Left => self.previous(),
            // KeyCode::Char('j') | KeyCode::Down => self.down(),
            // KeyCode::Char('k') | KeyCode::Up => self.up(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn info(&mut self) {}

    fn query(&mut self) {}
}

fn main() -> Result<()> {
    color_eyre::install()?;
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
