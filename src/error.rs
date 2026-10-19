use std::path::PathBuf;

use anyhow::Error;

pub fn user_error(path: Option<&PathBuf>, error: Error) -> String {
    match path {
        Some(path) => format!("prestui: error: {}: {error}", path.display()),
        None => format!("prestui: error: {error}"),
    }
}
