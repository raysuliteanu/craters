use color_eyre::Result;
use crates_io_api::{Crate, CrateResponse};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    app::{self},
    components::Component,
};

struct InfoPopup {
    title: String,
    content: Box<CrateResponse>,
}

impl InfoPopup {
    pub fn new(title: &str, content: Box<CrateResponse>) -> Self {
        Self {
            title: title.to_string(),
            content,
        }
    }

    fn render_info_popup(&mut self, frame: &mut Frame) {
        let lines = self.format_crate_info(&self.content.crate_data);
        let block = Block::bordered().bg(app::COLOR_BLACK).fg(Color::White);
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .fg(Color::White)
            .bg(app::COLOR_BLACK);
        let area = InfoPopup::popup_area(frame.area(), 60, 60);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }

    fn format_crate_info(&self, krate: &Crate) -> Vec<Line<'static>> {
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
                    .flat_map(|s| [Span::from(s).fg(Color::Green), Span::from("    ")])
                    .collect::<Vec<Span>>()
            })
            .unwrap_or_default();
        let name = Span::from(krate.name.clone()).bold().fg(Color::Cyan);
        let version = Span::from(krate.max_version.clone());
        vec![
            Line::from(vec![name, Span::from("    v"), version]),
            Line::from(""),
            Line::from(desc),
            Line::from(""),
            Line::from(tags),
        ]
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Component for InfoPopup {
    fn draw(&mut self, frame: &mut Frame, _area: Rect) -> Result<()> {
        self.render_info_popup(frame);
        Ok(())
    }
}
