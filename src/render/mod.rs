mod ansi;
mod ascii;
mod blocks;

use ::image::DynamicImage;
use ratatui::style::{Color, Style};

pub use ansi::AnsiRenderer;
pub use ascii::AsciiRenderer;
pub use blocks::BlocksRenderer;

#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub width: u32,
    pub height: Option<u32>,
    pub invert: bool,
    pub color: bool,
    pub charset: String,
    pub aspect_correct: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 80,
            height: None,
            invert: false,
            color: true,
            charset: "@%#*+=-:. ".into(),
            aspect_correct: true,
        }
    }
}

pub trait Renderer {
    fn render(&self, image: &DynamicImage, options: &RenderOptions) -> String;
}

pub(super) fn character_ramp(charset: &str) -> Vec<char> {
    if charset.is_empty() {
        "@%#*+=-:. ".chars().collect()
    } else {
        charset.chars().collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderCell {
    pub symbol: char,
    pub foreground: Option<Rgb>,
    pub background: Option<Rgb>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderGrid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<RenderCell>,
}

impl RenderGrid {
    pub fn cell(&self, x: u16, y: u16) -> RenderCell {
        self.cells[usize::from(y) * usize::from(self.width) + usize::from(x)]
    }

    pub fn style(cell: RenderCell) -> Style {
        let mut style = Style::default();
        if let Some(Rgb(r, g, b)) = cell.foreground {
            style = style.fg(Color::Rgb(r, g, b));
        }
        if let Some(Rgb(r, g, b)) = cell.background {
            style = style.bg(Color::Rgb(r, g, b));
        }
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewSize {
    pub width: u16,
    pub height: u16,
}

pub fn fit_preview_size(
    image: &DynamicImage,
    max_width: u16,
    max_height: u16,
    aspect_correct: bool,
    scale_percent: u16,
) -> PreviewSize {
    let max_width = max_width.max(1);
    let max_height = max_height.max(1);
    let cell_aspect = if aspect_correct { 0.5 } else { 1.0 };
    let source_ratio = image.width().max(1) as f32 / image.height().max(1) as f32;
    let scaled_width = (f32::from(max_width) * f32::from(scale_percent.min(100)) / 100.0)
        .round()
        .max(1.0) as u16;
    let natural_height = (f32::from(scaled_width) * cell_aspect / source_ratio)
        .ceil()
        .max(1.0) as u16;
    if natural_height <= max_height {
        PreviewSize {
            width: scaled_width,
            height: natural_height,
        }
    } else {
        let width = (f32::from(max_height) * source_ratio / cell_aspect)
            .floor()
            .max(1.0) as u16;
        PreviewSize {
            width: width.min(max_width),
            height: max_height,
        }
    }
}

pub fn render_cells(
    image: &DynamicImage,
    mode: crate::cli::RenderMode,
    options: &RenderOptions,
    max_width: u16,
    max_height: u16,
    scale_percent: u16,
) -> RenderGrid {
    let size = fit_preview_size(
        image,
        max_width,
        max_height,
        options.aspect_correct,
        scale_percent,
    );
    render_cells_for_size(image, mode, options, size)
}

/// Render at an explicit terminal-cell width. The resulting height is allowed
/// to exceed the viewport so callers can export the complete artwork.
pub fn render_cells_at_width(
    image: &DynamicImage,
    mode: crate::cli::RenderMode,
    options: &RenderOptions,
    width: u16,
) -> RenderGrid {
    let width = width.clamp(1, 1000);
    let source_ratio = image.width().max(1) as f32 / image.height().max(1) as f32;
    let cell_aspect = if options.aspect_correct { 0.5 } else { 1.0 };
    let height = (f32::from(width) * cell_aspect / source_ratio)
        .ceil()
        .clamp(1.0, 10_000.0) as u16;
    render_cells_for_size(image, mode, options, PreviewSize { width, height })
}

fn render_cells_for_size(
    image: &DynamicImage,
    mode: crate::cli::RenderMode,
    options: &RenderOptions,
    size: PreviewSize,
) -> RenderGrid {
    let vertical_resolution = if mode == crate::cli::RenderMode::Blocks {
        2
    } else {
        1
    };
    let resized = crate::image::resize::resize_for_cells(
        image,
        u32::from(size.width),
        Some(u32::from(size.height)),
        if options.aspect_correct { 0.5 } else { 1.0 },
        vertical_resolution,
    )
    .to_rgba8();
    let ramp = character_ramp(&options.charset);
    let mut cells = Vec::with_capacity(usize::from(size.width) * usize::from(size.height));
    for y in 0..size.height {
        for x in 0..size.width {
            let top = rgba(*resized.get_pixel(u32::from(x), u32::from(y) * vertical_resolution));
            let cell = match mode {
                crate::cli::RenderMode::Ascii => {
                    let alpha = f32::from(top.3) / 255.0;
                    let mut value = luminance(top.0, top.1, top.2) * alpha + (1.0 - alpha);
                    if options.invert {
                        value = 1.0 - value;
                    }
                    let index = (value * (ramp.len().saturating_sub(1) as f32)).round() as usize;
                    RenderCell {
                        symbol: ramp[index.min(ramp.len() - 1)],
                        foreground: Some(opaque_rgb(top)),
                        background: None,
                    }
                }
                crate::cli::RenderMode::Ansi => RenderCell {
                    symbol: '█',
                    foreground: Some(opaque_rgb(top)),
                    background: None,
                },
                crate::cli::RenderMode::Blocks => {
                    let bottom = if y * 2 + 1 < size.height * 2 {
                        rgba(*resized.get_pixel(u32::from(x), u32::from(y * 2 + 1)))
                    } else {
                        (0, 0, 0, 0)
                    };
                    RenderCell {
                        symbol: '▀',
                        foreground: Some(opaque_rgb(top)),
                        background: Some(opaque_rgb(bottom)),
                    }
                }
            };
            cells.push(cell);
        }
    }
    RenderGrid {
        width: size.width,
        height: size.height,
        cells,
    }
}

pub fn luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)) / 255.0
}

pub fn rgba(pixel: image::Rgba<u8>) -> (u8, u8, u8, u8) {
    (pixel[0], pixel[1], pixel[2], pixel[3])
}

fn opaque_rgb(pixel: (u8, u8, u8, u8)) -> Rgb {
    if pixel.3 < 16 {
        Rgb(0, 0, 0)
    } else {
        Rgb(pixel.0, pixel.1, pixel.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgba, RgbaImage};

    #[test]
    fn fit_never_exceeds_preview_and_preserves_landscape_shape() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(200, 100));
        let size = fit_preview_size(&image, 60, 10, true, 100);
        assert_eq!(
            size,
            PreviewSize {
                width: 40,
                height: 10
            }
        );
    }

    #[test]
    fn structured_blocks_pair_top_and_bottom_pixels() {
        let mut image = RgbaImage::new(1, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        let grid = render_cells(
            &image.into(),
            crate::cli::RenderMode::Blocks,
            &RenderOptions::default(),
            1,
            1,
            100,
        );
        assert_eq!(
            grid.cell(0, 0),
            RenderCell {
                symbol: '▀',
                foreground: Some(Rgb(255, 0, 0)),
                background: Some(Rgb(0, 0, 255))
            }
        );
    }

    #[test]
    fn ascii_cells_keep_the_sampled_foreground_color() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([12, 34, 56, 255]));
        let grid = render_cells(
            &image.into(),
            crate::cli::RenderMode::Ascii,
            &RenderOptions::default(),
            1,
            1,
            100,
        );
        assert_eq!(grid.cell(0, 0).foreground, Some(Rgb(12, 34, 56)));
    }

    #[test]
    fn explicit_width_is_preserved_for_complete_exports() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(2, 1));
        let grid = render_cells_at_width(
            &image,
            crate::cli::RenderMode::Ascii,
            &RenderOptions::default(),
            37,
        );
        assert_eq!(grid.width, 37);
        assert_eq!(
            grid.cells.len(),
            usize::from(grid.width) * usize::from(grid.height)
        );
    }
}
