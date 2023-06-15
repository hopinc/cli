use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn home_path(to_join: &str) -> Result<PathBuf> {
    let path = dirs::home_dir()
        .context("Could not find `home` directory")?
        .join(to_join);

    log::debug!("Home path + joined: {:?}", path);

    Ok(path)
}
