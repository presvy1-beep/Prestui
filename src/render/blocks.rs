use ::image::DynamicImage;

use super::{rgba, RenderOptions, Renderer};
use crate::image::resize::resize_for_cells;

pub struct BlocksRenderer;
type Rgb = (u8, u8, u8);

impl Renderer for BlocksRenderer {
    fn render(&self, image: &DynamicImage, options: &RenderOptions) -> String {
        let resized = resize_for_cells(
            image,
            options.width,
            options.height,
            if options.aspect_correct { 0.5 } else { 1.0 },
            2,
        )
        .to_rgba8();
        let mut out = String::new();
        for y in (0..resized.height()).step_by(2) {
            let mut previous: Option<(Rgb, Rgb)> = None;
            for x in 0..resized.width() {
                let top = rgba(*resized.get_pixel(x, y));
                let bottom = if y + 1 < resized.height() {
                    rgba(*resized.get_pixel(x, y + 1))
                } else {
                    (0, 0, 0, 0)
                };
                let top_rgb = if top.3 < 16 {
                    (0, 0, 0)
                } else {
                    (top.0, top.1, top.2)
                };
                let bottom_rgb = if bottom.3 < 16 {
                    (0, 0, 0)
                } else {
                    (bottom.0, bottom.1, bottom.2)
                };
                if options.color && previous != Some((top_rgb, bottom_rgb)) {
                    out.push_str(&format!(
                        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
                        top_rgb.0, top_rgb.1, top_rgb.2, bottom_rgb.0, bottom_rgb.1, bottom_rgb.2
                    ));
                    previous = Some((top_rgb, bottom_rgb));
                }
                out.push('▀');
            }
            if options.color {
                out.push_str("\x1b[0m");
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgba, RgbaImage};

    #[test]
    fn emits_half_block_and_resets_colors() {
        let mut image = RgbaImage::new(1, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        let options = RenderOptions {
            width: 1,
            aspect_correct: false,
            ..Default::default()
        };
        let output = BlocksRenderer.render(&DynamicImage::ImageRgba8(image), &options);
        assert!(output.contains("▀"));
        assert!(output.ends_with("\x1b[0m\n"));
    }
}
