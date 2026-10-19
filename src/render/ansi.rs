use ::image::DynamicImage;

use super::{rgba, RenderOptions, Renderer};
use crate::image::resize::resize_for_cells;

pub struct AnsiRenderer;

impl Renderer for AnsiRenderer {
    fn render(&self, image: &DynamicImage, options: &RenderOptions) -> String {
        let resized = resize_for_cells(
            image,
            options.width,
            options.height,
            if options.aspect_correct { 0.5 } else { 1.0 },
            1,
        )
        .to_rgba8();
        if !options.color {
            return super::AsciiRenderer.render(image, options);
        }
        let mut out = String::new();
        let mut previous = None;
        for y in 0..resized.height() {
            for x in 0..resized.width() {
                let (r, g, b, a) = rgba(*resized.get_pixel(x, y));
                let color = if a < 16 { (0, 0, 0) } else { (r, g, b) };
                if previous != Some(color) {
                    out.push_str(&format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2));
                    previous = Some(color);
                }
                out.push('█');
            }
            out.push_str("\x1b[0m\n");
            previous = None;
        }
        out
    }
}
