mod parse;
pub mod types;
pub mod util;

use anyhow::Result;
use clap::Parser;

use self::util::{check_version, download, swap_executables, unpack};
use crate::commands::update::types::Version;
use crate::commands::update::util::now_secs;
use crate::config::VERSION;
use crate::state::http::HttpClient;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update Hop to the latest version")]
pub struct Options {
    #[clap(short = 'f', long = "force", help = "Force update")]
    pub force: bool,

    #[clap(short = 'b', long = "beta", help = "Update to beta version")]
    pub beta: bool,
}

pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let http = HttpClient::new(None, None);

    let (update, version) = check_version(&Version::from_string(VERSION)?, options.beta).await?;

    if !update && !options.force {
        log::info!("CLI is up to date");
        return Ok(());
    }

    log::info!("Found new version {version} (current: {VERSION})");

    // download the new release
    let packed_temp = download(http, version.to_string())
        .await
        .expect("Failed to download");

    // unpack the new release
    let unpacked = unpack(packed_temp).await?;

    // swap the executables
    swap_executables(std::env::current_exe()?, unpacked).await?;

    state.ctx.last_version_check = Some((now_secs().to_string(), version.to_string()));
    state.ctx.save().await?;

    log::info!("Updated to {version}");

    Ok(())
}
