use std::path::Path;

use ::image::DynamicImage;
use anyhow::{bail, Context, Result};

pub fn load(path: &Path) -> Result<DynamicImage> {
    if !path.exists() {
        bail!("file not found");
    }
    ::image::open(path).with_context(|| "could not decode image")
}
