use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tokio::fs;

use super::checker::{check_version, now_secs};
use super::types::Version;
use super::util::{
    create_completions_commands, download, execute_commands, swap_exe_command, unpack,
    HOP_CLI_DOWNLOAD_URL,
};
use crate::config::{ARCH, VERSION};
use crate::state::http::HttpClient;
use crate::state::State;
use crate::store::Store;
use crate::utils::capitalize;

#[derive(Debug, Parser)]
#[clap(about = "Update Hop to the latest version")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "Force update")]
    pub force: bool,

    #[clap(short, long, help = "Update to beta version (if available)")]
    pub beta: bool,
}

#[cfg(feature = "update")]
pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let http = HttpClient::new(None, None);

    let (update, version) = check_version(&Version::from_string(VERSION)?, options.beta).await?;

    if !update && !options.force {
        log::info!("CLI is up to date");
        return Ok(());
    }

    log::info!("Found new version {version} (current: {VERSION})");

    let platform = capitalize(&sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string()));

    // download the new release
    let packed_temp = download(
        &http,
        HOP_CLI_DOWNLOAD_URL,
        &format!("v{version}"),
        &format!("hop-{ARCH}-{platform}"),
    )
    .await?;

    // unpack the new release
    let unpacked = unpack(&packed_temp, "hop").await?;

    // remove the tarball since it's no longer needed
    fs::remove_file(packed_temp).await?;

    let mut non_elevated_args: Vec<OsString> = vec![];
    let mut elevated_args: Vec<OsString> = vec![];

    let mut current = std::env::current_exe()?
        .canonicalize()?
        .to_string_lossy()
        .to_string();

    if current.starts_with(r"\\\\?\\") {
        current = current[7..].to_string();
    } else if current.starts_with(r"\\?\") {
        current = current[4..].to_string();
    }

    let current = PathBuf::from(current);

    log::debug!("Current executable: {current:?}");

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

    state.ctx.last_version_check = Some((now_secs()?.to_string(), version.to_string()));
    state.ctx.save().await?;

    log::info!("Updated to {version}");

    Ok(())
}
