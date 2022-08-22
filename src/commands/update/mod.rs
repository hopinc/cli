mod parse;
pub mod types;
pub mod util;

use anyhow::Result;
use clap::Parser;

use self::types::Version;
use self::util::{
    check_version, create_completions_commands, download, execute_commands, now_secs,
    swap_exe_command, unpack,
};
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

    let mut non_elevated_args: Vec<String> = vec![];
    let mut elevated_args: Vec<String> = vec![];

    let current = std::env::current_exe()?;

    // swap the executables
    swap_exe_command(
        &mut non_elevated_args,
        &mut elevated_args,
        current.clone(),
        unpacked,
    )
    .await;

    // create completions
    create_completions_commands(&mut non_elevated_args, &mut elevated_args, current).await;

    // execute the commands
    execute_commands(&non_elevated_args, &elevated_args).await?;

    state.ctx.last_version_check = Some((now_secs().to_string(), version.to_string()));
    state.ctx.save().await?;

    log::info!("Updated to {version}");

    Ok(())
}
