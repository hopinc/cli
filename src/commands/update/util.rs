use std::env::temp_dir;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command as Cmd;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, ensure, Result};
use runas::Command as SudoCmd;
use tokio::fs;

use super::types::{GithubRelease, Version};
use crate::config::VERSION;
use crate::state::http::HttpClient;
use crate::store::context::Context;
use crate::util::is_writable;

pub const RELEASE_HOP_CLI_URL: &str = "https://api.github.com/repos/hopinc/hop_cli/releases";
pub const HOP_CLI_DOWNLOAD_URL: &str = "https://github.com/hopinc/hop_cli/releases/download";

pub async fn check_version(current: &Version, beta: bool) -> Result<(bool, Version)> {
    let http = HttpClient::new(None, None);

    let response = http
        .client
        .get(RELEASE_HOP_CLI_URL)
        .send()
        .await
        .map_err(|_| anyhow!("Failed to get latest release"))?;

    ensure!(
        response.status().is_success(),
        "Failed to get latest release from Github: {}",
        response.status()
    );

    let data = response
        .json::<Vec<GithubRelease>>()
        .await
        .map_err(|_| anyhow!("Failed to parse Github release"))?;

    let latest = if beta {
        // the latest release that can be prereleased
        data
            .iter()
            // skip drafts
            .find(|r| !r.draft)
            .map(|r| r.tag_name.clone())
            .ok_or_else(|| anyhow!("No prerelease found"))?
    } else {
        // the latest release that is not prereleased
        data
            .iter()
            // skip drafts and prereleases
            .find(|r| !r.prerelease && !r.draft)
            .map(|r| r.tag_name.clone())
            .ok_or_else(|| anyhow!("No release found"))?
    };

    let latest = Version::from_string(&latest)?;

    if latest.is_newer_than(current) {
        Ok((true, latest))
    } else {
        Ok((false, current.clone()))
    }
}

// static time to check for updates
const HOUR_IN_SECONDS: u64 = 60 * 60;

pub async fn version_notice(mut ctx: Context) -> Result<()> {
    let now = now_secs();

    let last_check = ctx
        .last_version_check
        .clone()
        .map(|(time, version)| (time.parse::<u64>().unwrap_or(now), version));

    let (last_checked, last_newest) = match last_check {
        Some(data) => data,
        // more than an hour to force check
        None => (now - HOUR_IN_SECONDS - 1, VERSION.to_string()),
    };

    let last_newest = Version::from_string(&last_newest)?;
    let current = Version::from_string(VERSION)?;

    let new_version = if now - last_checked > HOUR_IN_SECONDS {
        let (update, latest) = check_version(&current, false)
            .await
            .unwrap_or((false, current));

        ctx.last_version_check = Some((now.to_string(), latest.to_string()));
        ctx.save().await?;

        if !update {
            return Ok(());
        }

        latest
    } else if last_newest.is_newer_than(&current) {
        last_newest
    } else {
        // skip fs action
        return Ok(());
    };

    log::warn!("A new version is available: {new_version}");

    #[cfg(feature = "update")]
    log::warn!("Use `{}` to update", ctx.update_command());

    Ok(())
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(not(windows))]
const COMPRESSED_FILE_EXTENSION: &str = "tar.gz";

#[cfg(windows)]
const COMPRESSED_FILE_EXTENSION: &str = "zip";

pub async fn download(
    http: &HttpClient,
    base_url: &str,
    version: &str,
    filename: &str,
) -> Result<PathBuf> {
    log::info!("Downloading {filename}@{version}");

    let response = http
        .client
        .get(&format!(
            "{base_url}/{version}/{filename}.{COMPRESSED_FILE_EXTENSION}"
        ))
        .send()
        .await
        .expect("Failed to get latest release");

    assert!(
        response.status().is_success(),
        "Failed to get latest release: {}",
        response.status()
    );

    let data = response
        .bytes()
        .await
        .expect("Failed to get latest release");

    let packed_temp = temp_dir().join(filename);

    log::debug!("Downloading to: {packed_temp:?}");

    fs::write(&packed_temp, &data).await?;

    Ok(packed_temp)
}

#[cfg(not(windows))]
pub async fn unpack(packed_temp: &PathBuf, filename: &str) -> Result<PathBuf> {
    use async_compression::tokio::bufread::GzipDecoder;
    use tokio::io::BufReader;
    use tokio_tar::Archive;

    let file = fs::File::open(packed_temp).await?;
    let reader = BufReader::new(file);
    let gunzip = GzipDecoder::new(reader);
    let mut tar = Archive::new(gunzip);

    let unpack_dir = temp_dir().join(format!("extract-tmp-{filename}"));

    // clean up any existing unpacked files
    fs::remove_dir_all(unpack_dir.clone()).await.ok();
    fs::create_dir_all(unpack_dir.clone()).await?;

    tar.unpack(&unpack_dir).await?;

    let exe = unpack_dir.join(filename);

    log::debug!("Unpacked to: {exe:?}");

    Ok(exe)
}

#[cfg(not(windows))]
pub async fn swap_exe_command(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    old_exe: PathBuf,
    new_exe: PathBuf,
) {
    if is_writable(&old_exe).await {
        non_elevated_args
    } else {
        elevated_args
    }
    .push(format!("mv {} {}", new_exe.display(), old_exe.display()).into());
}

// disable on macos because its doesnt allow to edit completions like this
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub async fn create_completions_commands(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    exe_path: PathBuf,
) {
    let command = format!(
        "&& mkdir -p /usr/share/zsh/site-functions && {} completions zsh > /usr/share/zsh/site-functions/_hop 2> /dev/null && chmod 644 /usr/share/zsh/site-functions/_hop",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/zsh/site-functions/_hop")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };

    let command = format!(
        "&& mkdir -p /usr/share/fish/completions && {} completions fish > /usr/share/fish/completions/hop.fish 2> /dev/null && chmod 644 /usr/share/fish/completions/hop.fish",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/fish/completions/hop.fish")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };

    let command = format!(
        "&& mkdir -p /usr/share/bash-completion/completions && {} completions bash > /usr/share/bash-completion/completions/hop 2> /dev/null && chmod 644 /usr/share/bash-completion/completions/hop",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/bash-completion/completions/hop")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };
}

#[cfg(windows)]
pub async fn unpack(packed_temp: &PathBuf, filename: &str) -> Result<PathBuf> {
    use async_zip::read::stream::ZipFileReader;

    log::debug!("Unpacking: {packed_temp:?}");

    let stream = fs::File::open(packed_temp).await?;
    // seeking breaks the zips since its a single file
    let mut zip = ZipFileReader::new(stream);

    let exe = temp_dir().join(&format!("{filename}.exe"));

    // unpack the only file
    let data = zip
        .entry_reader()
        .await?
        .expect("brokey entry")
        .read_to_end_crc()
        .await?;

    fs::write(&exe, &data).await?;

    log::debug!("Unpacked to: {exe:?}");

    Ok(exe)
}

#[cfg(windows)]
pub async fn swap_exe_command(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    old_exe: PathBuf,
    new_exe: PathBuf,
) {
    let temp_delete = temp_dir().join(".hop.tmp");

    if is_writable(&old_exe).await {
        non_elevated_args
    } else {
        elevated_args
    }
    .extend(vec![
        "del".into(),
        temp_delete.clone().into(),
        "2> nul".into(),
        "| true".into(),
        "&".into(),
        "move".into(),
        old_exe.clone().into(),
        temp_delete.clone().into(),
        "&".into(),
        "move".into(),
        new_exe.clone().into(),
        old_exe.clone().into(),
        "&".into(),
        "del".into(),
        temp_delete.clone().into(),
        "2> nul".into(),
        "| true".into(),
    ]);
}

// is windows autocomplete even supported?
#[cfg(any(target_os = "windows", target_os = "macos"))]
#[inline]
pub async fn create_completions_commands(
    _non_elevated_args: &mut [OsString],
    _elevated_args: &mut [OsString],
    _exe_path: PathBuf,
) {
}

#[cfg(windows)]
const CMD: &str = "cmd.exe";
#[cfg(not(windows))]
const CMD: &str = "sh";

#[cfg(windows)]
const CMD_ARGS: &[&str] = &["/C"];
#[cfg(not(windows))]
const CMD_ARGS: &[&str] = &["-c"];

pub async fn execute_commands(
    non_elevated_args: &Vec<OsString>,
    elevated_args: &Vec<OsString>,
) -> Result<()> {
    if !elevated_args.is_empty() {
        log::debug!("elevated commands: {elevated_args:?}");

        SudoCmd::new(CMD)
            .args(CMD_ARGS)
            .args(elevated_args)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to execute the command"))?;
    }

    if !non_elevated_args.is_empty() {
        log::debug!("non-elevated commands: {non_elevated_args:?}");

        Cmd::new(CMD)
            .args(CMD_ARGS)
            .args(non_elevated_args)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to execute the command"))?;
    }

    Ok(())
}
