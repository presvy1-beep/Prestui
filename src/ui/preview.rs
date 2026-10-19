use ratatui::{prelude::*, widgets::*};

use crate::render::{RenderGrid, RenderOptions};

pub fn draw_preview(frame: &mut Frame<'_>, area: Rect, grid: Option<&RenderGrid>, title: &str) {
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Rgb(70, 78, 92)))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(grid) = grid else {
        let message = Paragraph::new("No image loaded\n\nPress o to open an image.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(message, inner);
        return;
    };
    let offset_x = inner.width.saturating_sub(grid.width) / 2;
    let offset_y = inner.height.saturating_sub(grid.height) / 2;
    let buffer = frame.buffer_mut();
    for y in 0..grid.height.min(inner.height) {
        for x in 0..grid.width.min(inner.width) {
            let cell = grid.cell(x, y);
            let target = (inner.x + offset_x + x, inner.y + offset_y + y);
            if let Some(buffer_cell) = buffer.cell_mut(target) {
                let mut symbol = [0; 4];
                buffer_cell.set_symbol(cell.symbol.encode_utf8(&mut symbol));
                buffer_cell.set_style(RenderGrid::style(cell));
            }
        }
    }
}

pub fn draw_sidebar(
    frame: &mut Frame<'_>,
    area: Rect,
    mode: crate::cli::RenderMode,
    options: &RenderOptions,
    manual_width: Option<u16>,
) {
    let mode_name = |candidate| if candidate == mode { "●" } else { "○" };
    let width = manual_width.map_or_else(|| "Auto".to_string(), |width| width.to_string());
    let text = vec![
        Line::from(Span::styled(
            "RENDER",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "{} ASCII",
            mode_name(crate::cli::RenderMode::Ascii)
        )),
        Line::from(format!("{} ANSI", mode_name(crate::cli::RenderMode::Ansi))),
        Line::from(format!(
            "{} Blocks",
            mode_name(crate::cli::RenderMode::Blocks)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "SETTINGS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Width     {width}")),
        Line::from(format!("Aspect    {}", on_off(options.aspect_correct))),
        Line::from(format!("Invert    {}", on_off(options.invert))),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().border_style(Style::default().fg(Color::Rgb(55, 62, 74)))),
        area,
    );
}

fn on_off(value: bool) -> &'static str {
    if value {
        "On"
    } else {
        "Off"
    }
}
