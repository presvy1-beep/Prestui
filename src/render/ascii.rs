use ::image::DynamicImage;

use super::{character_ramp, luminance, RenderOptions, Renderer};
use crate::image::resize::resize_for_cells;

pub struct AsciiRenderer;

impl Renderer for AsciiRenderer {
    fn render(&self, image: &DynamicImage, options: &RenderOptions) -> String {
        let ramp = character_ramp(&options.charset);
        let resized = resize_for_cells(
            image,
            options.width,
            options.height,
            if options.aspect_correct { 0.5 } else { 1.0 },
            1,
        )
        .to_rgba8();
        let mut out = String::new();
        for y in 0..resized.height() {
            for x in 0..resized.width() {
                let p = resized.get_pixel(x, y);
                let alpha = f32::from(p[3]) / 255.0;
                let mut value = luminance(p[0], p[1], p[2]) * alpha + (1.0 - alpha);
                if options.invert {
                    value = 1.0 - value;
                }
                let index = (value * (ramp.len().saturating_sub(1) as f32)).round() as usize;
                out.push(ramp[index.min(ramp.len() - 1)]);
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
    fn maps_black_and_white_to_ramp_ends() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([255, 255, 255, 255]));
        let options = RenderOptions {
            width: 2,
            aspect_correct: false,
            ..Default::default()
        };
        assert_eq!(
            AsciiRenderer.render(&DynamicImage::ImageRgba8(image), &options),
            "@ \n"
        );
    }
}
