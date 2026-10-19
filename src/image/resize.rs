use ::image::{imageops::FilterType, DynamicImage};

pub fn resize_for_cells(
    image: &DynamicImage,
    width: u32,
    height: Option<u32>,
    cell_aspect: f32,
    vertical_resolution: u32,
) -> DynamicImage {
    let width = width.max(1);
    let source_ratio = image.width().max(1) as f32 / image.height().max(1) as f32;
    let natural_height = (width as f32 * cell_aspect / source_ratio).ceil() as u32;
    let height = height.unwrap_or(natural_height).max(1);
    let pixel_height = height.saturating_mul(vertical_resolution).max(1);
    image.resize_exact(width, pixel_height, FilterType::Triangle)
}

#[cfg(test)]
mod tests {
    use super::resize_for_cells;
    use image::{DynamicImage, RgbImage};

    #[test]
    fn preserves_source_ratio_with_cell_correction() {
        let source = DynamicImage::ImageRgb8(RgbImage::new(200, 100));
        let resized = resize_for_cells(&source, 80, None, 0.5, 1);
        assert_eq!(resized.width(), 80);
        assert_eq!(resized.height(), 20);
    }
}
