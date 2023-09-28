use anyhow::{Context, Result};
use clap::Parser;

use super::copy::fslike::FsLike;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Backup files from a deployment to local machine")]
#[group(skip)]
pub struct Options {
    #[clap(help = "Deployment name or id")]
    pub source: String,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let source = FsLike::from_str(&state, &format!("{}:/", options.source)).await?;

    let backup_file = dirs::download_dir()
        .or(dirs::home_dir().map(|home| home.join("Downloads")))
        .context("Could not find a download directory")?
        .join(format!(
            "hop-backup_{}_{}.tar.gz",
            options.source,
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        ))
        .to_string_lossy()
        .to_string();

    let (_, data) = source.read().await?;

    tokio::fs::write(&backup_file, data)
        .await
        .with_context(|| format!("Could not write to {backup_file}"))?;

    log::info!("Backup saved to {backup_file}");

    Ok(())
}
