use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, prelude::*, widgets::Paragraph, Terminal};

use crate::{
    cli::RenderMode,
    image,
    render::{
        render_cells, render_cells_at_width, AnsiRenderer, AsciiRenderer, BlocksRenderer,
        RenderGrid, RenderOptions, Renderer,
    },
    ui,
};

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Screen {
    Landing,
    Viewer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionOutcome {
    Cancelled,
    Loaded,
    Invalid,
}

pub fn run(image_path: Option<PathBuf>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, image_path);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("could not enable terminal raw mode")?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide) {
        disable_raw_mode().ok();
        return Err(error).context("could not enter alternate screen");
    }
    Terminal::new(CrosstermBackend::new(stdout)).context("could not initialize terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .context("could not restore terminal")?;
    terminal.show_cursor().context("could not show cursor")
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    image_path: Option<PathBuf>,
) -> Result<()> {
    let mut screen = initial_screen(image_path.as_deref());
    let mut loaded = match image_path {
        Some(path) => Some((path.clone(), image::load(&path)?)),
        None => None,
    };
    let mut mode = RenderMode::Blocks;
    let mut options = RenderOptions::default();
    let mut manual_width: Option<u16> = None;
    let mut show_help = false;
    let mut cached_key = None;
    let mut cached_grid: Option<RenderGrid> = None;
    let mut status: Option<String> = None;

    loop {
        let size = terminal.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let viewer = screen == Screen::Viewer;
        let wide = viewer && size.width >= 80;
        let header_height = if viewer { 1 } else { 0 };
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(header_height),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let (sidebar, preview) = if wide {
            let [sidebar, preview] =
                Layout::horizontal([Constraint::Length(20), Constraint::Min(1)]).areas(body);
            (Some(sidebar), preview)
        } else {
            (None, body)
        };
        let too_small = viewer && (size.width < 60 || size.height < 20);
        let key = loaded.as_ref().map(|(path, image)| {
            (
                path.clone(),
                image.width(),
                image.height(),
                preview.width,
                preview.height,
                mode,
                manual_width,
                options.aspect_correct,
                options.invert,
            )
        });
        if !too_small && key != cached_key {
            cached_grid = loaded.as_ref().map(|(_, image)| {
                if let Some(width) = manual_width {
                    render_cells_at_width(image, mode, &options, width)
                } else {
                    render_cells(
                        image,
                        mode,
                        &options,
                        preview.width.saturating_sub(2),
                        preview.height.saturating_sub(2),
                        100,
                    )
                }
            });
            cached_key = key;
        }

        terminal.draw(|frame| {
            if screen == Screen::Landing {
                ui::draw_landing(frame, body, status.as_deref());
            } else if too_small {
                let message = Paragraph::new("Terminal too small\n\nResize to at least 60×20")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(message, body);
            } else if let Some(sidebar) = sidebar {
                frame.render_widget(
                    Paragraph::new(header_text(loaded.as_ref())).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    header,
                );
                ui::draw_sidebar(frame, sidebar, mode, &options, manual_width);
                ui::draw_preview(frame, preview, cached_grid.as_ref(), mode_label(mode));
            } else {
                frame.render_widget(
                    Paragraph::new(header_text(loaded.as_ref())).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    header,
                );
                ui::draw_preview(frame, preview, cached_grid.as_ref(), mode_label(mode));
            }
            if screen == Screen::Viewer {
                ui::draw_footer(frame, footer, status.as_deref());
            }
            if show_help && screen == Screen::Viewer {
                ui::draw_help(frame);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if screen == Screen::Landing {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Enter | KeyCode::Char('o') => {
                            if let Some(path) = choose_image(terminal)? {
                                match apply_selection(Some(path), &mut loaded, &mut screen) {
                                    SelectionOutcome::Loaded => {
                                        status = None;
                                        cached_key = None;
                                    }
                                    SelectionOutcome::Invalid => {
                                        status = Some("Could not decode the selected image".into());
                                    }
                                    SelectionOutcome::Cancelled => {}
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('1') => mode = RenderMode::Ascii,
                    KeyCode::Char('2') => mode = RenderMode::Ansi,
                    KeyCode::Char('3') => mode = RenderMode::Blocks,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let current =
                            manual_width.or_else(|| cached_grid.as_ref().map(|grid| grid.width));
                        manual_width = Some(current.unwrap_or(1).saturating_add(10).min(1000));
                        cached_key = None;
                    }
                    KeyCode::Char('-') => {
                        let current =
                            manual_width.or_else(|| cached_grid.as_ref().map(|grid| grid.width));
                        manual_width = Some(current.unwrap_or(1).saturating_sub(10).max(1));
                        cached_key = None;
                    }
                    KeyCode::Char('i') => options.invert = !options.invert,
                    KeyCode::Char('a') => options.aspect_correct = !options.aspect_correct,
                    KeyCode::Char('o') => {
                        if let Some(path) = choose_image(terminal)? {
                            match apply_selection(Some(path), &mut loaded, &mut screen) {
                                SelectionOutcome::Loaded => {
                                    status = None;
                                    cached_key = None;
                                }
                                SelectionOutcome::Invalid => {
                                    status = Some("Could not decode the selected image".into());
                                }
                                SelectionOutcome::Cancelled => {}
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        if let Some((_, image)) = &loaded {
                            let width = manual_width
                                .or_else(|| cached_grid.as_ref().map(|grid| grid.width))
                                .unwrap_or(1);
                            let mut export_options = options.clone();
                            export_options.width = u32::from(width);
                            export_options.height = manual_width
                                .is_none()
                                .then(|| cached_grid.as_ref().map(|grid| u32::from(grid.height)))
                                .flatten();
                            let output = render_cli_text(image, mode, &export_options);
                            match copy_to_clipboard(mode, output) {
                                Ok(()) => {
                                    status = Some(format!(
                                        "Copied {} to clipboard",
                                        mode_label(mode).trim()
                                    ))
                                }
                                Err(error) => status = Some(format!("Copy failed: {error}")),
                            }
                        }
                    }
                    KeyCode::Char('?') => show_help = !show_help,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn header_text(loaded: Option<&(PathBuf, image::DynamicImage)>) -> String {
    match loaded {
        Some((path, image)) => format!(
            " PRESTUI                                      {} • {}×{}",
            path.display(),
            image.width(),
            image.height()
        ),
        None => {
            " PRESTUI                                      Images, rendered in your terminal".into()
        }
    }
}

fn initial_screen(image_path: Option<&std::path::Path>) -> Screen {
    if image_path.is_some() {
        Screen::Viewer
    } else {
        Screen::Landing
    }
}

fn apply_selection(
    path: Option<PathBuf>,
    loaded: &mut Option<(PathBuf, image::DynamicImage)>,
    screen: &mut Screen,
) -> SelectionOutcome {
    let Some(path) = path else {
        return SelectionOutcome::Cancelled;
    };
    match image::load(&path) {
        Ok(decoded) => {
            *loaded = Some((path, decoded));
            *screen = Screen::Viewer;
            SelectionOutcome::Loaded
        }
        Err(_) => SelectionOutcome::Invalid,
    }
}

fn choose_image(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<Option<PathBuf>> {
    suspend_terminal(terminal)?;
    let selected = rfd::FileDialog::new()
        .set_title("Select image")
        .add_filter("Images", SUPPORTED_IMAGE_EXTENSIONS)
        .pick_file();
    resume_terminal(terminal)?;
    Ok(selected)
}

fn suspend_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("could not suspend terminal raw mode")?;
    if let Err(error) = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    ) {
        enable_raw_mode().ok();
        return Err(error).context("could not suspend terminal screen");
    }
    Ok(())
}

fn resume_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    enable_raw_mode().context("could not restore terminal raw mode")?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        crossterm::cursor::Hide
    )
    .context("could not restore terminal screen")?;
    terminal.clear().context("could not redraw terminal")
}

fn render_cli_text(
    image: &image::DynamicImage,
    mode: RenderMode,
    options: &RenderOptions,
) -> String {
    match mode {
        RenderMode::Ascii => AsciiRenderer.render(image, options),
        RenderMode::Ansi => AnsiRenderer.render(image, options),
        RenderMode::Blocks => BlocksRenderer.render(image, options),
    }
}

fn copy_to_clipboard(mode: RenderMode, output: String) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("could not access system clipboard")?;
    clipboard
        .set_text(output)
        .with_context(|| format!("could not copy {mode:?} output"))?;
    Ok(())
}

fn mode_label(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Ascii => " ASCII ",
        RenderMode::Ansi => " ANSI ",
        RenderMode::Blocks => " BLOCKS ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn starts_on_landing_without_an_image() {
        assert_eq!(initial_screen(None), Screen::Landing);
    }

    #[test]
    fn direct_image_argument_starts_in_viewer() {
        assert_eq!(initial_screen(Some(Path::new("image.png"))), Screen::Viewer);
    }

    #[test]
    fn canceling_selection_preserves_landing() {
        let mut loaded = None;
        let mut screen = Screen::Landing;
        assert_eq!(
            apply_selection(None, &mut loaded, &mut screen),
            SelectionOutcome::Cancelled
        );
        assert_eq!(screen, Screen::Landing);
        assert!(loaded.is_none());
    }

    #[test]
    fn valid_selection_transitions_landing_to_viewer() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("image.png");
        ::image::RgbImage::new(2, 2)
            .save(&path)
            .expect("write image fixture");
        let mut loaded = None;
        let mut screen = Screen::Landing;
        assert_eq!(
            apply_selection(Some(path.clone()), &mut loaded, &mut screen),
            SelectionOutcome::Loaded
        );
        assert_eq!(screen, Screen::Viewer);
        assert_eq!(
            loaded.as_ref().map(|(loaded_path, _)| loaded_path),
            Some(&path)
        );
    }

    #[test]
    fn canceling_viewer_selection_preserves_current_image() {
        let current_path = PathBuf::from("current.png");
        let mut loaded = Some((current_path.clone(), ::image::DynamicImage::new_rgb8(1, 1)));
        let mut screen = Screen::Viewer;
        assert_eq!(
            apply_selection(None, &mut loaded, &mut screen),
            SelectionOutcome::Cancelled
        );
        assert_eq!(screen, Screen::Viewer);
        assert_eq!(loaded.as_ref().map(|(path, _)| path), Some(&current_path));
    }

    #[test]
    fn invalid_selection_does_not_crash_or_change_screen() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("not-an-image.txt");
        fs::write(&path, "not an image").expect("write fixture");
        let mut loaded = None;
        let mut screen = Screen::Landing;
        assert_eq!(
            apply_selection(Some(path), &mut loaded, &mut screen),
            SelectionOutcome::Invalid
        );
        assert_eq!(screen, Screen::Landing);
        assert!(loaded.is_none());
    }

    #[test]
    fn landing_uses_full_logo_only_when_it_fits() {
        assert_eq!(ui::landing_logo(ui::wordmark_width()), ui::PRESTUI_WORDMARK);
        assert_eq!(
            ui::landing_logo(ui::wordmark_width() - 1),
            ui::COMPACT_WORDMARK
        );
    }
}
