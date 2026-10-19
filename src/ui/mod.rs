mod help;
mod preview;

pub use help::draw_help;
pub use preview::{draw_preview, draw_sidebar};

use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

pub const PRESTUI_WORDMARK: &str = r"██████╗ ██████╗ ███████╗███████╗████████╗██╗   ██╗██╗
██╔══██╗██╔══██╗██╔════╝██╔════╝╚══██╔══╝██║   ██║██║
██████╔╝██████╔╝█████╗  ███████╗   ██║   ██║   ██║██║
██╔═══╝ ██╔══██╗██╔══╝  ╚════██║   ██║   ██║   ██║██║
██║     ██║  ██║███████╗███████║   ██║   ╚██████╔╝██║
╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝";

pub const COMPACT_WORDMARK: &str = "PRESTUI";

pub fn wordmark_width() -> u16 {
    PRESTUI_WORDMARK
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u16
}

pub fn draw_footer(frame: &mut Frame<'_>, area: Rect, status: Option<&str>) {
    if let Some(status) = status {
        frame.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::Yellow)),
            area,
        );
        return;
    }
    let footer = Line::from(vec![
        Span::styled(" 1 ASCII ", Style::default().fg(Color::Cyan)),
        Span::styled(" 2 ANSI ", Style::default().fg(Color::Cyan)),
        Span::styled(" 3 Blocks ", Style::default().fg(Color::Cyan)),
        Span::raw(" +/- Size   o Open   s Copy   ? Help   q Quit"),
    ]);
    frame.render_widget(Paragraph::new(footer), area);
}

pub fn draw_landing(frame: &mut Frame<'_>, area: Rect, status: Option<&str>) {
    if area.width < COMPACT_WORDMARK.len() as u16 || area.height < 10 {
        frame.render_widget(
            Paragraph::new("Terminal too small\nResize to continue")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            area,
        );
        return;
    }

    let logo = landing_logo(area.width);
    let logo_height = logo.lines().count() as u16;
    let button_width = 18;
    let status_height = if status.is_some() { 1 } else { 0 };
    let content_height = logo_height + 1 + 1 + 2 + 3 + 1 + 1 + status_height;
    let top = area.y + area.height.saturating_sub(content_height) / 2;
    let logo_area = Rect::new(area.x, top, area.width, logo_height);
    frame.render_widget(
        Paragraph::new(logo).alignment(Alignment::Center).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        logo_area,
    );

    let subtitle_area = Rect::new(area.x, logo_area.bottom() + 1, area.width, 1);
    frame.render_widget(
        Paragraph::new("Images, rendered for the terminal.").alignment(Alignment::Center),
        subtitle_area,
    );

    let button_area = Rect::new(
        area.x + area.width.saturating_sub(button_width) / 2,
        subtitle_area.bottom() + 2,
        button_width.min(area.width),
        3,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " Select Image ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(Block::bordered().border_style(Style::default().fg(Color::Cyan))),
        button_area,
    );

    let hint_area = Rect::new(area.x, button_area.bottom() + 1, area.width, 1);
    frame.render_widget(
        Paragraph::new("Enter Select   o Select Image   q Quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        hint_area,
    );

    if let Some(status) = status {
        let status_area = Rect::new(area.x, hint_area.bottom() + 1, area.width, 1);
        frame.render_widget(
            Paragraph::new(status)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            status_area,
        );
    }
}

pub fn landing_logo(width: u16) -> &'static str {
    if width >= wordmark_width() {
        PRESTUI_WORDMARK
    } else {
        COMPACT_WORDMARK
    }
}
