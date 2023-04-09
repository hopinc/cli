mod fslike;
mod utils;

use anyhow::{bail, Result};
use clap::Parser;

use crate::state::State;

use self::fslike::FsLike;

#[derive(Debug, Parser)]
#[clap(about = "Copy files between volumes and local machine")]
pub struct Options {
    #[clap(help = "Source to copy from")]
    pub source: String,
    #[clap(help = "Target to copy to")]
    pub target: String,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let source = FsLike::from_str(&state, &options.source).await?;
    let target = FsLike::from_str(&state, &options.target).await?;

    // because users could just use `cp` to copy files between local directories
    if source.is_local() && target.is_local() {
        bail!("Specify at least one remote path");
    }

    // temporary limitation
    if !source.is_local() && !target.is_local() {
        bail!("Specify at least one local path");
    }

    let transfer_size = source.to(target).await?;

    log::info!(
        "Copied from {} to {} ({} bytes)",
        options.source,
        options.target,
        transfer_size
    );

    Ok(())
}
