use ratatui::{prelude::*, widgets::*};

pub fn draw_help(frame: &mut Frame<'_>) {
    let area = centered_rect(70, 60, frame.area());
    let text = "Prestui controls\n\n1 / 2 / 3   ASCII / ANSI / Blocks\n+ / -       Change preview width\ni           Invert luminance\na           Toggle aspect correction\no           Open image picker\ns           Copy preview\n?           Toggle this help\nq / Esc     Quit";
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(" Help "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - width) / 2),
        Constraint::Percentage(width),
        Constraint::Percentage((100 - width) / 2),
    ])
    .split(area);
    Layout::vertical([
        Constraint::Percentage((100 - height) / 2),
        Constraint::Percentage(height),
        Constraint::Percentage((100 - height) / 2),
    ])
    .split(horizontal[1])[1]
}
